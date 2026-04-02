use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;

use crate::auth::jwt::{validate_token, Claims};
use crate::config::AppConfig;
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

        match token {
            Some(token) => match validate_token(config, token) {
                Ok(claims) => Outcome::Success(AuthenticatedUser { claims }),
                Err(_) => Outcome::Error((
                    Status::Unauthorized,
                    Json(ApiError::unauthorized("Invalid or expired token")),
                )),
            },
            None => Outcome::Error((
                Status::Unauthorized,
                Json(ApiError::unauthorized("Missing authorization header")),
            )),
        }
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
