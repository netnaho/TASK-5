use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;

use crate::config::AppConfig;
use crate::dto::privacy::*;
use crate::middleware::auth_guard::{AuthenticatedUser, AdminGuard};
use crate::middleware::reauth_guard::ReauthRequired;
use crate::services::privacy_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[post("/requests", data = "<body>")]
pub async fn create_request(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    body: Json<CreateDataRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    let uuid = privacy_service::create_data_request(pool.inner(), user.claims.user_id, &body)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

#[get("/requests?<status>")]
pub async fn list_requests(
    pool: &State<MySqlPool>,
    _user: AdminGuard,
    status: Option<String>,
) -> Result<Json<ApiResponse<Vec<DataRequestResponse>>>, (Status, Json<ApiError>)> {
    let requests = privacy_service::list_requests(pool.inner(), status.as_deref())
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(requests))
}

#[get("/requests/my")]
pub async fn my_requests(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<DataRequestResponse>>>, (Status, Json<ApiError>)> {
    let requests = privacy_service::list_user_requests(pool.inner(), user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(requests))
}

#[post("/requests/<uuid>/review", data = "<body>")]
pub async fn review_request(
    pool: &State<MySqlPool>,
    config: &State<AppConfig>,
    user: AdminGuard,
    _reauth: ReauthRequired,
    uuid: String,
    body: Json<AdminReviewDataRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    privacy_service::admin_review_request(pool.inner(), config.inner(), &uuid, &body, user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(if body.approved {
        "Request approved and processed".to_string()
    } else {
        "Request rejected".to_string()
    }))
}

#[post("/sensitive", data = "<body>")]
pub async fn store_sensitive(
    pool: &State<MySqlPool>,
    config: &State<AppConfig>,
    user: AuthenticatedUser,
    body: Json<StoreSensitiveDataRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    // Use the dedicated DATA_ENCRYPTION_KEY — validated as 64 hex chars at startup.
    // key_version = 2 identifies records encrypted with this dedicated key.
    privacy_service::store_sensitive_field(
        pool.inner(),
        user.claims.user_id,
        &body.field_name,
        &body.value,
        &config.data_encryption_key,
        2,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Sensitive data stored".to_string()))
}

#[get("/sensitive")]
pub async fn get_masked(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<MaskedFieldResponse>>>, (Status, Json<ApiError>)> {
    let fields = privacy_service::get_masked_fields(pool.inner(), user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(fields))
}

pub fn routes() -> Vec<Route> {
    routes![create_request, list_requests, my_requests, review_request, store_sensitive, get_masked]
}
