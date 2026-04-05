use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;
use validator::Validate;

use crate::config::AppConfig;
use crate::dto::approval::*;
use crate::middleware::auth_guard::{CourseAuthorGuard, ReviewerGuard, AuthenticatedUser};
use crate::middleware::hmac_guard::HmacVerified;
use crate::middleware::reauth_guard::ReauthReviewerGuard;
use crate::services::approval_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[post("/<course_uuid>/submit", data = "<body>")]
pub async fn submit_for_approval(
    pool: &State<MySqlPool>,
    config: &State<AppConfig>,
    user: CourseAuthorGuard,
    course_uuid: String,
    body: Json<SubmitApprovalRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }

    let uuid = approval_service::submit_for_approval(
        pool.inner(), config.inner(), &course_uuid, &body, user.claims.user_id, &user.claims.role, None,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok(serde_json::json!({"approval_uuid": uuid})))
}

#[post("/<approval_uuid>/review", data = "<body>")]
pub async fn review_approval(
    pool: &State<MySqlPool>,
    config: &State<AppConfig>,
    user: ReauthReviewerGuard,
    approval_uuid: String,
    body: Json<ReviewApprovalRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    approval_service::review_approval(
        pool.inner(), config.inner(), &approval_uuid, &body,
        user.claims.user_id, &user.claims.role, None,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    let msg = if body.approved { "Approval step approved".to_string() } else { "Approval rejected".to_string() };
    Ok(ApiResponse::ok(msg))
}

#[get("/<uuid>")]
pub async fn get_approval(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    uuid: String,
) -> Result<Json<ApiResponse<ApprovalResponse>>, (Status, Json<ApiError>)> {
    let approval = approval_service::get_approval(
        pool.inner(), &uuid, &user.claims.role, user.claims.user_id, user.claims.department_id,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(approval))
}

#[get("/queue")]
pub async fn approval_queue(
    pool: &State<MySqlPool>,
    user: ReviewerGuard,
) -> Result<Json<ApiResponse<Vec<ApprovalQueueItem>>>, (Status, Json<ApiError>)> {
    let items = approval_service::list_approval_queue(pool.inner(), &user.claims.role, user.claims.department_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(items))
}

#[post("/process-scheduled")]
pub async fn process_scheduled(
    pool: &State<MySqlPool>,
    _hmac: HmacVerified,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    let count = approval_service::process_scheduled_transitions(pool.inner())
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"transitions_processed": count})))
}

#[post("/<course_uuid>/unpublish", data = "<body>")]
pub async fn submit_for_unpublish(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    course_uuid: String,
    body: Json<SubmitApprovalRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }

    let uuid = approval_service::submit_for_unpublish(
        pool.inner(), &course_uuid, &body, user.claims.user_id, &user.claims.role, None,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok(serde_json::json!({"approval_uuid": uuid})))
}

pub fn routes() -> Vec<Route> {
    routes![submit_for_approval, submit_for_unpublish, review_approval, get_approval, approval_queue, process_scheduled]
}
