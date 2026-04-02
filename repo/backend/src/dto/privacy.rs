use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDataRequest {
    pub request_type: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminReviewDataRequest {
    pub approved: bool,
    pub admin_notes: Option<String>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct StoreSensitiveDataRequest {
    #[validate(length(min = 1))]
    pub field_name: String,
    #[validate(length(min = 1))]
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct MaskedFieldResponse {
    pub field_name: String,
    pub masked_value: String,
}
