use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskRule {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub conditions: serde_json::Value,
    pub severity: String,
    pub is_active: bool,
    pub created_by: i64,
    pub schedule_interval_minutes: i32,
    pub last_run_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskEvent {
    pub id: i64,
    pub uuid: String,
    pub rule_id: i64,
    pub user_id: Option<i64>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub risk_score: f64,
    pub details: Option<serde_json::Value>,
    pub status: String,
    pub reviewed_by: Option<i64>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub escalated_to: Option<i64>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlacklistedEmployer {
    pub id: i64,
    pub uuid: String,
    pub employer_name: String,
    pub reason: String,
    pub added_by: i64,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployerPosting {
    pub id: i64,
    pub uuid: String,
    pub employer_name: String,
    pub posting_type: String,
    pub title: String,
    pub description: Option<String>,
    pub compensation: Option<f64>,
    pub posted_by: i64,
    pub flagged: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub event_type: String,
    pub channel: String,
    pub target_url: Option<String>,
    pub signing_secret: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
