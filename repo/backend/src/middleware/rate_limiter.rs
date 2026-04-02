use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use sqlx::MySqlPool;

use crate::config::AppConfig;
use crate::middleware::auth_guard::AuthenticatedUser;
use crate::repositories::rate_limit_repo;
use crate::utils::errors::ApiError;

pub struct RateLimited {
    pub claims: crate::auth::jwt::Claims,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimited {
    type Error = Json<ApiError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match AuthenticatedUser::from_request(req).await {
            Outcome::Success(u) => u,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        let pool = req.rocket().state::<MySqlPool>().expect("DB pool not configured");
        let config = req.rocket().state::<AppConfig>().expect("AppConfig not configured");

        let count = rate_limit_repo::increment_request_count(pool, user.claims.user_id)
            .await
            .unwrap_or(0);

        if count > config.rate_limit_per_minute as i32 {
            return Outcome::Error((
                Status::TooManyRequests,
                Json(ApiError::new(Status::TooManyRequests, "Rate limit exceeded. Try again in a minute.")),
            ));
        }

        Outcome::Success(RateLimited { claims: user.claims })
    }
}
