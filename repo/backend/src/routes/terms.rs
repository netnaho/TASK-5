use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;

use crate::dto::term::{TermResponse, TermAcceptanceResponse};
use crate::middleware::auth_guard::AuthenticatedUser;
use crate::repositories::term_repo;
use crate::services::term_service;
use crate::utils::errors::{ApiError, AppError};
use crate::utils::response::ApiResponse;

fn to_response(t: crate::models::term::Term) -> TermResponse {
    TermResponse {
        uuid: t.uuid,
        name: t.name,
        code: t.code,
        start_date: t.start_date.to_string(),
        end_date: t.end_date.to_string(),
        is_active: t.is_active,
    }
}

#[get("/")]
pub async fn list_terms(
    pool: &State<MySqlPool>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<TermResponse>>>, (Status, Json<ApiError>)> {
    let terms = term_repo::list_terms(pool.inner()).await
        .map_err(AppError::Database)?;
    Ok(ApiResponse::ok(terms.into_iter().map(to_response).collect()))
}

#[get("/active")]
pub async fn get_active_term(
    pool: &State<MySqlPool>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Option<TermResponse>>>, (Status, Json<ApiError>)> {
    let term = term_repo::find_active_term(pool.inner()).await
        .map_err(AppError::Database)?;
    Ok(ApiResponse::ok(term.map(to_response)))
}

#[post("/<term_uuid>/accept")]
pub async fn accept_term(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    term_uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    term_service::accept_term(pool.inner(), &term_uuid, user.claims.user_id, None, None)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Term accepted".to_string()))
}

#[get("/my-acceptances")]
pub async fn my_acceptances(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<TermAcceptanceResponse>>>, (Status, Json<ApiError>)> {
    let acceptances = term_repo::get_user_acceptances(pool.inner(), user.claims.user_id)
        .await.map_err(AppError::Database)?;
    let responses = acceptances.into_iter().map(|a| TermAcceptanceResponse {
        uuid: a.uuid,
        term_id: a.term_id,
        accepted_at: a.accepted_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }).collect();
    Ok(ApiResponse::ok(responses))
}

pub fn routes() -> Vec<Route> {
    routes![list_terms, get_active_term, accept_term, my_acceptances]
}
