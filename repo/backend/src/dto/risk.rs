use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct RiskRuleResponse {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub severity: String,
    pub is_active: bool,
    pub schedule_interval_minutes: i32,
    pub last_run_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RiskEventResponse {
    pub uuid: String,
    pub rule_id: i64,
    pub rule_name: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub risk_score: f64,
    pub details: Option<serde_json::Value>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRiskEventRequest {
    pub status: String,
    pub notes: Option<String>,
    pub escalate_to: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubscriptionRequest {
    #[validate(length(min = 1))]
    pub event_type: String,
    pub channel: Option<String>,
    /// Required when channel = "webhook". Must be an approved on-prem URL.
    pub target_url: Option<String>,
    /// Optional HMAC-SHA256 signing secret. When absent no signature header is sent.
    pub signing_secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub uuid: String,
    pub event_type: String,
    pub channel: String,
    pub is_active: bool,
    /// Present only for webhook subscriptions; signing_secret is never returned.
    pub target_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostingRequest {
    #[validate(length(min = 1))]
    pub employer_name: String,
    pub posting_type: String,
    #[validate(length(min = 1))]
    pub title: String,
    pub description: Option<String>,
    pub compensation: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddBlacklistRequest {
    #[validate(length(min = 1))]
    pub employer_name: String,
    #[validate(length(min = 1))]
    pub reason: String,
}
