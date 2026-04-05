use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use sqlx::MySqlPool;

use crate::auth::jwt::{validate_token, Claims};
use crate::config::AppConfig;
use crate::repositories::rate_limit_repo;
use crate::utils::errors::ApiError;

pub struct AuthenticatedUser {
    pub claims: Claims,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = Json<ApiError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = req.rocket().state::<AppConfig>().expect("AppConfig not configured");

        let token = req.headers().get_one("Authorization")
            .and_then(|header| header.strip_prefix("Bearer "));

        let claims = match token {
            Some(token) => match validate_token(config, token) {
                Ok(claims) => claims,
                Err(_) => return Outcome::Error((
                    Status::Unauthorized,
                    Json(ApiError::unauthorized("Invalid or expired token")),
                )),
            },
            None => return Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Missing authorization header")),
            )),
        };

        // Rate limiting: applied to every authenticated request in one place.
        // All role guards delegate here, so no per-route changes are needed.
        let pool = req.rocket().state::<MySqlPool>().expect("DB pool not configured");
        let count = rate_limit_repo::increment_request_count(pool, claims.user_id)
            .await
            .unwrap_or(0);
        if count > config.rate_limit_per_minute as i32 {
            return Outcome::Error((
                Status::TooManyRequests,
                Json(ApiError::new(
                    Status::TooManyRequests,
                    "Rate limit exceeded. Try again in a minute.",
                )),
            ));
        }

        Outcome::Success(AuthenticatedUser { claims })
    }
}

// Role-specific guards
macro_rules! impl_role_guard {
    ($name:ident, $role:expr) => {
        pub struct $name {
            pub claims: Claims,
        }

        #[rocket::async_trait]
        impl<'r> FromRequest<'r> for $name {
            type Error = Json<ApiError>;

            async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
                let user = match AuthenticatedUser::from_request(req).await {
                    Outcome::Success(u) => u,
                    Outcome::Error(e) => return Outcome::Error(e),
                    Outcome::Forward(f) => return Outcome::Forward(f),
                };

                if user.claims.role == $role || user.claims.role == "admin" {
                    Outcome::Success(Self { claims: user.claims })
                } else {
                    Outcome::Error((
                        Status::Forbidden,
                        Json(ApiError::forbidden("Insufficient permissions")),
                    ))
                }
            }
        }
    };
}

impl_role_guard!(AdminGuard, "admin");
impl_role_guard!(StaffAuthorGuard, "staff_author");
impl_role_guard!(DeptReviewerGuard, "dept_reviewer");
impl_role_guard!(FacultyGuard, "faculty");

// A guard that allows any of: admin, staff_author (course authors)
pub struct CourseAuthorGuard {
    pub claims: Claims,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CourseAuthorGuard {
    type Error = Json<ApiError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match AuthenticatedUser::from_request(req).await {
            Outcome::Success(u) => u,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        match user.claims.role.as_str() {
            "admin" | "staff_author" => Outcome::Success(Self { claims: user.claims }),
            _ => Outcome::Error((
                Status::Forbidden,
                Json(ApiError::forbidden("Only course authors and admins can perform this action")),
            )),
        }
    }
}

// A guard that allows reviewers (dept_reviewer, admin)
pub struct ReviewerGuard {
    pub claims: Claims,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ReviewerGuard {
    type Error = Json<ApiError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match AuthenticatedUser::from_request(req).await {
            Outcome::Success(u) => u,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        match user.claims.role.as_str() {
            "admin" | "dept_reviewer" => Outcome::Success(Self { claims: user.claims }),
            _ => Outcome::Error((
                Status::Forbidden,
                Json(ApiError::forbidden("Only reviewers and admins can perform this action")),
            )),
        }
    }
}
