use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub uuid: String,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub entity_type: Option<String>,
    pub entity_uuid: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}
