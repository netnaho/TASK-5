use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Course {
    pub id: i64,
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
    pub release_notes: Option<String>,
    pub effective_date: Option<NaiveDateTime>,
    pub updated_on: Option<NaiveDateTime>,
    pub current_version: i32,
    pub created_by: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CourseSection {
    pub id: i64,
    pub uuid: String,
    pub course_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_published: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Lesson {
    pub id: i64,
    pub uuid: String,
    pub section_id: i64,
    pub title: String,
    pub content_type: String,
    pub content_body: Option<String>,
    pub content_html: Option<String>,
    pub sort_order: i32,
    pub duration_minutes: Option<i32>,
    pub is_published: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaAsset {
    pub id: i64,
    pub uuid: String,
    pub lesson_id: Option<i64>,
    pub uploaded_by: i64,
    pub file_name: String,
    pub file_path: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub checksum: Option<String>,
    pub alt_text: Option<String>,
    pub status: String,
    pub validated: bool,
    pub validation_error: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CourseVersion {
    pub id: i64,
    pub uuid: String,
    pub course_id: i64,
    pub version_number: i32,
    pub snapshot: serde_json::Value,
    pub created_by: i64,
    pub change_summary: Option<String>,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VersionDiff {
    pub id: i64,
    pub uuid: String,
    pub course_id: i64,
    pub from_version: i32,
    pub to_version: i32,
    pub diff_data: serde_json::Value,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CourseStatus {
    Draft,
    PendingApproval,
    ApprovedScheduled,
    Published,
    Unpublished,
    Rejected,
}

impl CourseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseStatus::Draft => "draft",
            CourseStatus::PendingApproval => "pending_approval",
            CourseStatus::ApprovedScheduled => "approved_scheduled",
            CourseStatus::Published => "published",
            CourseStatus::Unpublished => "unpublished",
            CourseStatus::Rejected => "rejected",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(CourseStatus::Draft),
            "pending_approval" => Some(CourseStatus::PendingApproval),
            "approved_scheduled" => Some(CourseStatus::ApprovedScheduled),
            "published" => Some(CourseStatus::Published),
            "unpublished" => Some(CourseStatus::Unpublished),
            "rejected" => Some(CourseStatus::Rejected),
            _ => None,
        }
    }
}

pub const ALLOWED_MEDIA_TYPES: &[&str] = &["application/pdf", "video/mp4", "image/png"];
pub const MAX_MEDIA_SIZE_BYTES: i64 = 500 * 1024 * 1024; // 500 MB
