use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct ResourceResponse {
    pub uuid: String,
    pub name: String,
    pub resource_type: String,
    pub location: Option<String>,
    pub capacity: Option<i32>,
    pub description: Option<String>,
    pub open_time: String,
    pub close_time: String,
    pub max_booking_hours: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct AvailabilitySlot {
    pub start: String,
    pub end: String,
    pub available: bool,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBookingRequest {
    pub resource_uuid: String,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub uuid: String,
    pub resource_id: i64,
    pub resource_name: Option<String>,
    pub booked_by: i64,
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
    pub reschedule_count: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RescheduleRequest {
    pub new_start_time: String,
    pub new_end_time: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BreachResponse {
    pub uuid: String,
    pub user_id: Option<i64>,
    pub booking_id: Option<i64>,
    pub breach_type: String,
    pub severity: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct RestrictionResponse {
    pub uuid: String,
    pub user_id: i64,
    pub restriction_type: String,
    pub reason: String,
    pub starts_at: String,
    pub expires_at: Option<String>,
    pub is_active: bool,
    pub auto_triggered: bool,
}
