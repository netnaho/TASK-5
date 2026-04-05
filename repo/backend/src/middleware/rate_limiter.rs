// Rate limiting is enforced inside `AuthenticatedUser::from_request()`, which is the
// shared base for every authenticated guard (AdminGuard, CourseAuthorGuard, etc.).
// This means all authenticated endpoints are covered without per-route changes.
//
// `RateLimited` is kept as a named guard for call-sites that want to be explicit about
// rate-limit semantics. It delegates entirely to `AuthenticatedUser` — no second
// increment or check is performed here.

use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;

use crate::middleware::auth_guard::AuthenticatedUser;
use crate::utils::errors::ApiError;

pub struct RateLimited {
    pub claims: crate::auth::jwt::Claims,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimited {
    type Error = Json<ApiError>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthenticatedUser::from_request(req).await {
            Outcome::Success(u) => Outcome::Success(RateLimited { claims: u.claims }),
            Outcome::Error(e) => Outcome::Error(e),
            Outcome::Forward(f) => Outcome::Forward(f),
        }
    }
}
