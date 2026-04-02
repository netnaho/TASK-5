use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCourseRequest {
    #[validate(length(min = 1, max = 500, message = "Title is required and max 500 chars"))]
    pub title: String,
    #[validate(length(min = 1, max = 50, message = "Code is required and max 50 chars"))]
    pub code: String,
    pub description: Option<String>,
    pub department_id: Option<i64>,
    pub term_id: Option<i64>,
    pub max_enrollment: Option<i32>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCourseRequest {
    #[validate(length(min = 1, max = 500, message = "Title max 500 chars"))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub department_id: Option<i64>,
    pub term_id: Option<i64>,
    pub max_enrollment: Option<i32>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSectionRequest {
    #[validate(length(min = 1, max = 500, message = "Title is required"))]
    pub title: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSectionRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct SectionResponse {
    pub uuid: String,
    pub course_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_published: bool,
    pub lessons: Vec<LessonResponse>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLessonRequest {
    #[validate(length(min = 1, max = 500, message = "Title is required"))]
    pub title: String,
    pub content_type: Option<String>,
    pub content_body: Option<String>,
    pub content_html: Option<String>,
    pub sort_order: Option<i32>,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLessonRequest {
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub content_body: Option<String>,
    pub content_html: Option<String>,
    pub sort_order: Option<i32>,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMediaRequest {
    #[validate(length(min = 1, message = "File name is required"))]
    pub file_name: String,
    pub file_path: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub checksum: Option<String>,
    pub alt_text: Option<String>,
    pub lesson_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct MediaResponse {
    pub uuid: String,
    pub file_name: String,
    pub file_path: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub status: String,
    pub validated: bool,
    pub validation_error: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 100, message = "Tag name is required"))]
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TagResponse {
    pub id: i64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub uuid: String,
    pub version_number: i32,
    pub change_summary: Option<String>,
    pub snapshot: serde_json::Value,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pub from_version: i32,
    pub to_version: i32,
    pub diff_data: serde_json::Value,
}
