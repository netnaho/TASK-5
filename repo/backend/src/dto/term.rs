use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TermResponse {
    pub uuid: String,
    pub name: String,
    pub code: String,
    pub start_date: String,
    pub end_date: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct TermAcceptanceResponse {
    pub uuid: String,
    pub term_id: i64,
    pub accepted_at: String,
}
