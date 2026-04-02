use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApprovalRequest {
    pub id: i64,
    pub uuid: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub requested_by: i64,
    pub status: String,
    pub priority: String,
    pub notes: Option<String>,
    pub release_notes: Option<String>,
    pub effective_date: Option<NaiveDateTime>,
    pub version_number: Option<i32>,
    pub resolved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApprovalStep {
    pub id: i64,
    pub uuid: String,
    pub approval_request_id: i64,
    pub step_order: i32,
    pub reviewer_id: Option<i64>,
    pub reviewer_role: Option<String>,
    pub status: String,
    pub comments: Option<String>,
    pub decided_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledTransition {
    pub id: i64,
    pub uuid: String,
    pub course_id: i64,
    pub approval_request_id: i64,
    pub target_status: String,
    pub scheduled_at: NaiveDateTime,
    pub executed_at: Option<NaiveDateTime>,
    pub is_executed: bool,
    pub created_at: NaiveDateTime,
}
