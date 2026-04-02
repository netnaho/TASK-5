use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct SubmitApprovalRequest {
    #[validate(length(min = 1, message = "Release notes are required"))]
    pub release_notes: String,
    #[validate(length(min = 1, message = "Effective date is required (MM/DD/YYYY HH:MM AM/PM)"))]
    pub effective_date: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReviewApprovalRequest {
    pub approved: bool,
    pub comments: Option<String>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct ApprovalStepResponse {
    pub uuid: String,
    pub step_order: i32,
    pub reviewer_id: Option<i64>,
    pub reviewer_role: Option<String>,
    pub status: String,
    pub comments: Option<String>,
    pub decided_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApprovalQueueItem {
    pub approval: ApprovalResponse,
    pub course_title: String,
    pub course_code: String,
    pub requester_name: String,
}
