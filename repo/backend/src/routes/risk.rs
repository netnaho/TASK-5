use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;
use validator::Validate;

use crate::dto::risk::*;
use crate::middleware::auth_guard::{AuthenticatedUser, AdminGuard};
use crate::services::risk_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[get("/rules")]
pub async fn list_rules(
    pool: &State<MySqlPool>,
    _user: AdminGuard,
) -> Result<Json<ApiResponse<Vec<RiskRuleResponse>>>, (Status, Json<ApiError>)> {
    let rules = risk_service::list_rules(pool.inner())
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(rules))
}

#[get("/events?<limit>")]
pub async fn list_events(
    pool: &State<MySqlPool>,
    _user: AdminGuard,
    limit: Option<i64>,
) -> Result<Json<ApiResponse<Vec<RiskEventResponse>>>, (Status, Json<ApiError>)> {
    let events = risk_service::list_events(pool.inner(), limit)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(events))
}

#[put("/events/<uuid>", data = "<body>")]
pub async fn update_event(
    pool: &State<MySqlPool>,
    user: AdminGuard,
    uuid: String,
    body: Json<UpdateRiskEventRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    risk_service::update_event(pool.inner(), &uuid, &body, user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Risk event updated".to_string()))
}

#[post("/evaluate")]
pub async fn run_evaluation(
    pool: &State<MySqlPool>,
    _user: AdminGuard,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    let count = risk_service::run_risk_evaluation(pool.inner())
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"events_created": count})))
}

#[post("/subscriptions", data = "<body>")]
pub async fn create_subscription(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    body: Json<CreateSubscriptionRequest>,
) -> Result<Json<ApiResponse<SubscriptionResponse>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let sub = risk_service::create_subscription(pool.inner(), user.claims.user_id, &body.event_type, body.channel.as_deref())
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(sub))
}

#[get("/subscriptions")]
pub async fn list_subscriptions(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<SubscriptionResponse>>>, (Status, Json<ApiError>)> {
    let subs = risk_service::list_subscriptions(pool.inner(), user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(subs))
}

#[post("/postings", data = "<body>")]
pub async fn create_posting(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    body: Json<CreatePostingRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let uuid = risk_service::create_posting(pool.inner(), &body, user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

#[post("/blacklist", data = "<body>")]
pub async fn add_blacklist(
    pool: &State<MySqlPool>,
    _user: AdminGuard,
    body: Json<AddBlacklistRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let uuid = risk_service::add_blacklist(pool.inner(), &body, _user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

pub fn routes() -> Vec<Route> {
    routes![list_rules, list_events, update_event, run_evaluation,
            create_subscription, list_subscriptions, create_posting, add_blacklist]
}
