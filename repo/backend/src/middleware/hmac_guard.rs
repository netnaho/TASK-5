use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use sqlx::MySqlPool;

use crate::auth::hmac;
use crate::config::AppConfig;
use crate::utils::errors::ApiError;

pub struct HmacVerified {
    pub key_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HmacVerified {
    type Error = Json<ApiError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = req.rocket().state::<AppConfig>().expect("AppConfig not configured");
        let pool = req.rocket().state::<MySqlPool>().expect("DB pool not configured");

        let key_id = match req.headers().get_one("X-HMAC-Key-Id") {
            Some(k) => k.to_string(),
            None => return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Missing X-HMAC-Key-Id header")),
            )),
        };

        let nonce = match req.headers().get_one("X-HMAC-Nonce") {
            Some(n) => n.to_string(),
            None => return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Missing X-HMAC-Nonce header")),
            )),
        };

        let timestamp: i64 = match req.headers().get_one("X-HMAC-Timestamp").and_then(|t| t.parse().ok()) {
            Some(t) => t,
            None => return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Missing or invalid X-HMAC-Timestamp header")),
            )),
        };

        let signature = match req.headers().get_one("X-HMAC-Signature") {
            Some(s) => s.to_string(),
            None => return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Missing X-HMAC-Signature header")),
            )),
        };

        // Validate timestamp freshness
        if !hmac::is_timestamp_valid(timestamp, config.hmac_nonce_expiry_seconds) {
            return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Request timestamp expired")),
            ));
        }

        // Check nonce not reused
        let nonce_exists: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM used_nonces WHERE nonce = ?"
        ).bind(&nonce).fetch_optional(pool).await.unwrap_or(None);

        if nonce_exists.is_some() {
            return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Nonce already used (replay attack prevented)")),
            ));
        }

        // Look up HMAC key
        let key_row: Option<(String,)> = sqlx::query_as(
            "SELECT secret_hash FROM hmac_keys WHERE key_id = ? AND is_active = true AND (expires_at IS NULL OR expires_at > NOW())"
        ).bind(&key_id).fetch_optional(pool).await.unwrap_or(None);

        let secret = match key_row {
            Some((s,)) => s,
            None => return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Invalid or expired HMAC key")),
            )),
        };

        // Build message and verify
        let body_str = req.headers().get_one("X-HMAC-Body").unwrap_or("");
        let message = hmac::build_signing_message(&key_id, &nonce, timestamp, body_str);

        if !hmac::verify_signature(&secret, &message, &signature) {
            return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Invalid HMAC signature")),
            ));
        }

        // Store nonce to prevent replay
        let _ = sqlx::query(
            "INSERT INTO used_nonces (nonce, key_id, expires_at) VALUES (?, ?, DATE_ADD(NOW(), INTERVAL 5 MINUTE))"
        ).bind(&nonce).bind(&key_id).execute(pool).await;

        Outcome::Success(HmacVerified { key_id })
    }
}
