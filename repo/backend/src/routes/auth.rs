use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;
use validator::Validate;

use crate::config::AppConfig;
use crate::dto::auth::{LoginRequest, LoginResponse, ChangePasswordRequest, ReauthRequest};
use crate::middleware::auth_guard::AuthenticatedUser;
use crate::middleware::client_ip::ClientIp;
use crate::middleware::reauth_guard::{ReauthRequired, ReauthAdminGuard};
use crate::repositories::login_rate_limit_repo;
use crate::services::auth_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[post("/login", data = "<body>")]
pub async fn login(
    pool: &State<MySqlPool>,
    config: &State<AppConfig>,
    client_ip: ClientIp,
    body: Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, (Status, Json<ApiError>)> {
    if let Err(validation_errors) = body.validate() {
        let msg = validation_errors
            .field_errors()
            .values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>()
            .join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }

    // IP-based rate limiting for login
    let ip = &client_ip.0;
    let rate_exceeded = login_rate_limit_repo::check_ip_rate(
        pool.inner(), ip, "/auth/login",
        config.login_rate_limit_per_minute, config.login_rate_limit_per_hour,
    ).await.unwrap_or(false);
    if rate_exceeded {
        return Err((Status::TooManyRequests, Json(ApiError::new(
            Status::TooManyRequests, "Too many login attempts. Try again later.",
        ))));
    }
    let _ = login_rate_limit_repo::increment_ip_rate(pool.inner(), ip, "/auth/login").await;

    let result = auth_service::login(pool.inner(), config.inner(), &body.username, &body.password, Some(ip), None)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok(result))
}

#[post("/change-password", data = "<body>")]
pub async fn change_password(
    pool: &State<MySqlPool>,
    user: ReauthRequired,
    body: Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    if let Err(validation_errors) = body.validate() {
        let msg = validation_errors.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }

    auth_service::change_password(pool.inner(), user.claims.user_id, &body.current_password, &body.new_password, None, None)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok("Password changed successfully".to_string()))
}

#[post("/reauth", data = "<body>")]
pub async fn reauth(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    body: Json<ReauthRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    auth_service::reauth(pool.inner(), user.claims.user_id, &body.password, None, None)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok("Re-authentication successful".to_string()))
}

#[get("/me")]
pub async fn me(
    user: AuthenticatedUser,
    pool: &State<MySqlPool>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    use crate::repositories::user_repo;

    let u = user_repo::find_by_uuid(pool.inner(), &user.claims.sub)
        .await
        .map_err(|e| {
            let err: (Status, Json<ApiError>) = crate::utils::errors::AppError::Database(e).into();
            err
        })?
        .ok_or_else(|| {
            let err: (Status, Json<ApiError>) =
                crate::utils::errors::AppError::NotFound("User not found".to_string()).into();
            err
        })?;

    Ok(ApiResponse::ok(serde_json::json!({
        "uuid": u.uuid,
        "username": u.username,
        "email": u.email,
        "full_name": u.full_name,
        "role": u.role,
        "department_id": u.department_id,
        "is_active": u.is_active
    })))
}

#[derive(serde::Deserialize)]
pub struct CreateHmacKeyRequest {
    pub key_id: String,
    pub secret: String,
    pub description: Option<String>,
}

#[post("/hmac-keys", data = "<body>")]
pub async fn create_hmac_key(
    pool: &State<MySqlPool>,
    user: ReauthAdminGuard,
    body: Json<CreateHmacKeyRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    let uuid = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO hmac_keys (uuid, key_id, secret_hash, description, owner_user_id, is_active) VALUES (?, ?, ?, ?, ?, TRUE)"
    )
    .bind(&uuid).bind(&body.key_id).bind(&body.secret)
    .bind(body.description.as_deref()).bind(user.claims.user_id)
    .execute(pool.inner()).await
    .map_err(|e| {
        let err: (Status, Json<ApiError>) = crate::utils::errors::AppError::Database(e).into();
        err
    })?;

    Ok(ApiResponse::ok(serde_json::json!({"key_id": body.key_id, "uuid": uuid})))
}

pub fn routes() -> Vec<Route> {
    routes![login, change_password, reauth, me, create_hmac_key]
}
