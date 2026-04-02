use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use sqlx::MySqlPool;
use chrono::Utc;

use crate::config::AppConfig;
use crate::middleware::auth_guard::AuthenticatedUser;
use crate::repositories::user_repo;
use crate::utils::errors::ApiError;

/// Guard that requires recent re-authentication (within configured window, default 15 min)
pub struct ReauthRequired {
    pub claims: crate::auth::jwt::Claims,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ReauthRequired {
    type Error = Json<ApiError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match AuthenticatedUser::from_request(req).await {
            Outcome::Success(u) => u,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        let pool = req.rocket().state::<MySqlPool>().expect("DB pool not configured");
        let config = req.rocket().state::<AppConfig>().expect("AppConfig not configured");

        let db_user = match user_repo::find_by_id(pool, user.claims.user_id).await {
            Ok(Some(u)) => u,
            _ => return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("User not found")),
            )),
        };

        let reauth_ok = db_user.last_reauth_at.map(|ts| {
            let elapsed = Utc::now().naive_utc() - ts;
            elapsed.num_minutes() < config.reauth_window_minutes
        }).unwrap_or(false);

        if !reauth_ok {
            return Outcome::Error((
                Status::Forbidden,
                Json(ApiError::new(Status::Forbidden, "Re-authentication required for this action. POST /api/v1/auth/reauth first.")),
            ));
        }

        Outcome::Success(ReauthRequired { claims: user.claims })
    }
}
