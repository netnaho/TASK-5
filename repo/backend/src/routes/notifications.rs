use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;

use crate::dto::notification::{NotificationResponse, UnreadCountResponse};
use crate::middleware::auth_guard::AuthenticatedUser;
use crate::repositories::notification_repo;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[get("/")]
pub async fn list_notifications(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<NotificationResponse>>>, (Status, Json<ApiError>)> {
    let items = notification_repo::list_for_user(pool.inner(), user.claims.user_id, 50)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(crate::utils::errors::AppError::Database(e)))?;

    let result = items.into_iter().map(|n| NotificationResponse {
        uuid: n.uuid,
        title: n.title,
        message: n.message,
        notification_type: n.notification_type,
        entity_type: n.entity_type,
        entity_uuid: n.entity_uuid,
        is_read: n.is_read,
        created_at: n.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }).collect();

    Ok(ApiResponse::ok(result))
}

#[get("/unread-count")]
pub async fn unread_count(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<UnreadCountResponse>>, (Status, Json<ApiError>)> {
    let count = notification_repo::get_unread_count(pool.inner(), user.claims.user_id)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(crate::utils::errors::AppError::Database(e)))?;

    Ok(ApiResponse::ok(UnreadCountResponse { count }))
}

#[put("/<uuid>/read")]
pub async fn mark_read(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    notification_repo::mark_read(pool.inner(), &uuid, user.claims.user_id)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(crate::utils::errors::AppError::Database(e)))?;

    Ok(ApiResponse::ok("Notification marked as read".to_string()))
}

#[put("/read-all")]
pub async fn mark_all_read(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    notification_repo::mark_all_read(pool.inner(), user.claims.user_id)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(crate::utils::errors::AppError::Database(e)))?;

    Ok(ApiResponse::ok("All notifications marked as read".to_string()))
}

pub fn routes() -> Vec<Route> {
    routes![list_notifications, unread_count, mark_read, mark_all_read]
}
