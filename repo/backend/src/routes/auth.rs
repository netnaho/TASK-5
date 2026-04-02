use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;
use validator::Validate;

use crate::config::AppConfig;
use crate::dto::auth::{LoginRequest, LoginResponse, ChangePasswordRequest, ReauthRequest};
use crate::middleware::auth_guard::AuthenticatedUser;
use crate::services::auth_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[post("/login", data = "<body>")]
pub async fn login(
    pool: &State<MySqlPool>,
    config: &State<AppConfig>,
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

    let result = auth_service::login(pool.inner(), config.inner(), &body.username, &body.password, None, None)
        .await
        .map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok(result))
}

#[post("/change-password", data = "<body>")]
pub async fn change_password(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
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
) -> Result<Json<serde_json::Value>, (Status, Json<ApiError>)> {
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

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "uuid": u.uuid,
            "username": u.username,
            "email": u.email,
            "full_name": u.full_name,
            "role": u.role,
            "department_id": u.department_id,
            "is_active": u.is_active
        }
    })))
}

pub fn routes() -> Vec<Route> {
    routes![login, change_password, reauth, me]
}
