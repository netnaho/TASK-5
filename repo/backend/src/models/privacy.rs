use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PersonalDataRequest {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub request_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub processed_by: Option<i64>,
    pub processed_at: Option<NaiveDateTime>,
    pub approved_by: Option<i64>,
    pub approved_at: Option<NaiveDateTime>,
    pub admin_notes: Option<String>,
    pub result_file_path: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SensitiveDataVault {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub field_name: String,
    pub encrypted_value: String,
    pub iv: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
