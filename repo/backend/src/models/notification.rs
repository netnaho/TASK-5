use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub entity_type: Option<String>,
    pub entity_uuid: Option<String>,
    pub is_read: bool,
    pub created_at: NaiveDateTime,
}
