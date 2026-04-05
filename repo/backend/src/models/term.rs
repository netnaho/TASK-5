use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Term {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TermAcceptance {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub term_id: i64,
    pub accepted_at: chrono::NaiveDateTime,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
