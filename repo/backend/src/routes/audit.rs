use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;

use crate::middleware::auth_guard::AdminGuard;
use crate::services::audit_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[get("/?<entity_type>&<entity_id>&<limit>")]
pub async fn list_audit_logs(
    pool: &State<MySqlPool>,
    _user: AdminGuard,
    entity_type: Option<String>,
    entity_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Json<ApiResponse<Vec<crate::models::audit::AuditLog>>>, (Status, Json<ApiError>)> {
    let logs = audit_service::list_audit_logs(
        pool.inner(), entity_type.as_deref(), entity_id, limit,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(logs))
}

pub fn routes() -> Vec<Route> {
    routes![list_audit_logs]
}
