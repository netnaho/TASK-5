use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct UserInfo {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub department_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CourseResponse {
    pub uuid: String,
    pub title: String,
    pub code: String,
    pub description: Option<String>,
    pub department_id: Option<i64>,
    pub term_id: Option<i64>,
    pub instructor_id: Option<i64>,
    pub status: String,
    pub visibility: String,
    pub max_enrollment: Option<i32>,
    pub current_version: i32,
    pub release_notes: Option<String>,
    pub effective_date: Option<String>,
    pub updated_on: Option<String>,
    pub tags: Vec<TagResponse>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagResponse {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SectionResponse {
    pub uuid: String,
    pub course_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_published: bool,
    pub lessons: Vec<LessonResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LessonResponse {
    pub uuid: String,
    pub section_id: i64,
    pub title: String,
    pub content_type: String,
    pub content_body: Option<String>,
    pub content_html: Option<String>,
    pub sort_order: i32,
    pub duration_minutes: Option<i32>,
    pub is_published: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionResponse {
    pub uuid: String,
    pub version_number: i32,
    pub change_summary: Option<String>,
    pub snapshot: serde_json::Value,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalResponse {
    pub uuid: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub status: String,
    pub priority: String,
    pub release_notes: Option<String>,
    pub effective_date: Option<String>,
    pub version_number: Option<i32>,
    pub notes: Option<String>,
    pub requested_by: i64,
    pub steps: Vec<ApprovalStepResponse>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalStepResponse {
    pub uuid: String,
    pub step_order: i32,
    pub reviewer_id: Option<i64>,
    pub reviewer_role: Option<String>,
    pub status: String,
    pub comments: Option<String>,
    pub decided_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalQueueItem {
    pub approval: ApprovalResponse,
    pub course_title: String,
    pub course_code: String,
    pub requester_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AvailabilitySlot {
    pub start: String,
    pub end: String,
    pub available: bool,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscriptionResponse {
    pub uuid: String,
    pub event_type: String,
    pub channel: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataRequestResponse {
    pub uuid: String,
    pub user_id: i64,
    pub request_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub admin_notes: Option<String>,
    pub result_file_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaskedFieldResponse {
    pub field_name: String,
    pub masked_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditLogEntry {
    pub uuid: String,
    pub user_id: Option<i64>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub correlation_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UuidResponse {
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalUuidResponse {
    pub approval_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountResponse {
    pub events_created: Option<u32>,
    pub transitions_processed: Option<u32>,
}
