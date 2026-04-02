use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i64,
    pub uuid: String,
    pub user_id: Option<i64>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub correlation_id: Option<String>,
    pub retention_expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityEvent {
    pub id: i64,
    pub uuid: String,
    pub event_type: String,
    pub severity: String,
    pub user_id: Option<i64>,
    pub ip_address: Option<String>,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub correlation_id: Option<String>,
    pub created_at: NaiveDateTime,
}
