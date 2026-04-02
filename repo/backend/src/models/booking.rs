use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub resource_type: String,
    pub location: Option<String>,
    pub capacity: Option<i32>,
    pub description: Option<String>,
    pub open_time: chrono::NaiveTime,
    pub close_time: chrono::NaiveTime,
    pub max_booking_hours: i32,
    pub requires_approval: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Booking {
    pub id: i64,
    pub uuid: String,
    pub resource_id: i64,
    pub booked_by: i64,
    pub title: String,
    pub description: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub status: String,
    pub recurrence_rule: Option<String>,
    pub reschedule_count: i32,
    pub approved_by: Option<i64>,
    pub approved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BookingReschedule {
    pub id: i64,
    pub uuid: String,
    pub booking_id: i64,
    pub reschedule_number: i32,
    pub requested_by: i64,
    pub original_start: NaiveDateTime,
    pub original_end: NaiveDateTime,
    pub new_start: NaiveDateTime,
    pub new_end: NaiveDateTime,
    pub reason: Option<String>,
    pub status: String,
    pub decided_by: Option<i64>,
    pub decided_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceBlackout {
    pub id: i64,
    pub uuid: String,
    pub resource_id: i64,
    pub reason: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub created_by: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Breach {
    pub id: i64,
    pub uuid: String,
    pub user_id: Option<i64>,
    pub booking_id: Option<i64>,
    pub breach_type: String,
    pub severity: String,
    pub description: String,
    pub evidence: Option<serde_json::Value>,
    pub status: String,
    pub resolved_by: Option<i64>,
    pub resolved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Restriction {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub restriction_type: String,
    pub reason: String,
    pub imposed_by: i64,
    pub starts_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub is_active: bool,
    pub breach_count: i32,
    pub auto_triggered: bool,
    pub created_at: NaiveDateTime,
}

pub const MAX_RESCHEDULES: i32 = 2;
pub const MAX_ACTIVE_PER_RESOURCE: i32 = 2;
pub const MAX_ADVANCE_DAYS: i64 = 90;
pub const BREACH_WINDOW_DAYS: i64 = 60;
pub const BREACH_THRESHOLD: i64 = 3;
pub const LATE_CANCEL_HOURS: i64 = 2;
