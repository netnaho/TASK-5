use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;
use validator::Validate;

use crate::dto::course::{CreateTagRequest, TagResponse};
use crate::middleware::auth_guard::{AuthenticatedUser, CourseAuthorGuard};
use crate::services::course_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[post("/", data = "<body>")]
pub async fn create_tag(
    pool: &State<MySqlPool>,
    _user: CourseAuthorGuard,
    body: Json<CreateTagRequest>,
) -> Result<Json<ApiResponse<TagResponse>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let tag = course_service::create_tag(pool.inner(), &body.name)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(tag))
}

#[get("/")]
pub async fn list_tags(
    pool: &State<MySqlPool>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<TagResponse>>>, (Status, Json<ApiError>)> {
    let tags = course_service::list_tags(pool.inner())
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(tags))
}

pub fn routes() -> Vec<Route> {
    routes![create_tag, list_tags]
}
