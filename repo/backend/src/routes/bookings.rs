use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use sqlx::MySqlPool;
use validator::Validate;

use crate::dto::booking::*;
use crate::middleware::auth_guard::AuthenticatedUser;
use crate::services::booking_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[get("/resources")]
pub async fn list_resources(
    pool: &State<MySqlPool>,
    _user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<ResourceResponse>>>, (Status, Json<ApiError>)> {
    let resources = booking_service::list_resources(pool.inner())
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(resources))
}

#[get("/resources/<uuid>/availability?<date>")]
pub async fn resource_availability(
    pool: &State<MySqlPool>,
    _user: AuthenticatedUser,
    uuid: String,
    date: String,
) -> Result<Json<ApiResponse<Vec<AvailabilitySlot>>>, (Status, Json<ApiError>)> {
    let slots = booking_service::check_availability(pool.inner(), &uuid, &date)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(slots))
}

#[post("/", data = "<body>")]
pub async fn create_booking(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    body: Json<CreateBookingRequest>,
) -> Result<Json<ApiResponse<BookingResponse>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let booking = booking_service::create_booking(pool.inner(), &body, user.claims.user_id, None)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(booking))
}

#[post("/<uuid>/reschedule", data = "<body>")]
pub async fn reschedule_booking(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    uuid: String,
    body: Json<RescheduleRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    booking_service::reschedule_booking(pool.inner(), &uuid, &body, user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Booking rescheduled".to_string()))
}

#[post("/<uuid>/cancel")]
pub async fn cancel_booking(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    booking_service::cancel_booking(pool.inner(), &uuid, user.claims.user_id, None)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Booking cancelled".to_string()))
}

#[get("/my")]
pub async fn my_bookings(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<BookingResponse>>>, (Status, Json<ApiError>)> {
    let bookings = booking_service::list_user_bookings(pool.inner(), user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(bookings))
}

#[get("/breaches")]
pub async fn my_breaches(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<BreachResponse>>>, (Status, Json<ApiError>)> {
    let breaches = booking_service::list_breaches(pool.inner(), user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(breaches))
}

#[get("/restrictions")]
pub async fn my_restrictions(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<RestrictionResponse>>>, (Status, Json<ApiError>)> {
    let restrictions = booking_service::list_restrictions(pool.inner(), user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(restrictions))
}

pub fn routes() -> Vec<Route> {
    routes![list_resources, resource_availability, create_booking, reschedule_booking,
            cancel_booking, my_bookings, my_breaches, my_restrictions]
}
