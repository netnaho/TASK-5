use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
use rocket::form::Form;
use rocket::fs::TempFile;
use sqlx::MySqlPool;
use validator::Validate;

use crate::config::AppConfig;
use crate::dto::course::*;
use crate::middleware::auth_guard::{AuthenticatedUser, CourseAuthorGuard};
use crate::services::course_service;
use crate::utils::errors::ApiError;
use crate::utils::response::ApiResponse;

#[post("/", data = "<body>")]
pub async fn create_course(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    body: Json<CreateCourseRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }

    let uuid = course_service::create_course(pool.inner(), &body, user.claims.user_id, None)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

#[get("/")]
pub async fn list_courses(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<ApiResponse<Vec<CourseResponse>>>, (Status, Json<ApiError>)> {
    let courses = course_service::list_courses(
        pool.inner(), &user.claims.role, user.claims.user_id, user.claims.department_id,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok(courses))
}

#[get("/<uuid>")]
pub async fn get_course(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    uuid: String,
) -> Result<Json<ApiResponse<CourseResponse>>, (Status, Json<ApiError>)> {
    let course = course_service::get_course(
        pool.inner(), &uuid, &user.claims.role, user.claims.user_id, user.claims.department_id,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(course))
}

#[put("/<uuid>", data = "<body>")]
pub async fn update_course(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
    body: Json<UpdateCourseRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::update_course(pool.inner(), &uuid, &body, user.claims.user_id, &user.claims.role, None)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Course updated".to_string()))
}

#[delete("/<uuid>")]
pub async fn delete_course(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::delete_course(pool.inner(), &uuid, user.claims.user_id, &user.claims.role, None)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Course deleted".to_string()))
}

// --- Sections ---
#[post("/<course_uuid>/sections", data = "<body>")]
pub async fn create_section(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    course_uuid: String,
    body: Json<CreateSectionRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let uuid = course_service::create_section(
        pool.inner(), &course_uuid, &body, user.claims.user_id, &user.claims.role,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

#[get("/<course_uuid>/sections")]
pub async fn list_sections(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    course_uuid: String,
) -> Result<Json<ApiResponse<Vec<SectionResponse>>>, (Status, Json<ApiError>)> {
    let sections = course_service::list_sections_with_lessons(
        pool.inner(), &course_uuid, &user.claims.role, user.claims.user_id, user.claims.department_id,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(sections))
}

#[put("/sections/<uuid>", data = "<body>")]
pub async fn update_section(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
    body: Json<UpdateSectionRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::update_section(pool.inner(), &uuid, &body, user.claims.user_id, &user.claims.role)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Section updated".to_string()))
}

#[delete("/sections/<uuid>")]
pub async fn delete_section(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::delete_section(pool.inner(), &uuid, user.claims.user_id, &user.claims.role)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Section deleted".to_string()))
}

// --- Lessons ---
#[post("/sections/<section_uuid>/lessons", data = "<body>")]
pub async fn create_lesson(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    section_uuid: String,
    body: Json<CreateLessonRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let uuid = course_service::create_lesson(
        pool.inner(), &section_uuid, &body, user.claims.user_id, &user.claims.role,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

#[put("/lessons/<uuid>", data = "<body>")]
pub async fn update_lesson(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
    body: Json<UpdateLessonRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::update_lesson(pool.inner(), &uuid, &body, user.claims.user_id, &user.claims.role)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Lesson updated".to_string()))
}

#[delete("/lessons/<uuid>")]
pub async fn delete_lesson(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::delete_lesson(pool.inner(), &uuid, user.claims.user_id, &user.claims.role)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Lesson deleted".to_string()))
}

// --- Media ---

#[derive(FromForm)]
pub struct MediaUpload<'f> {
    file: TempFile<'f>,
    alt_text: Option<String>,
    lesson_id: Option<i64>,
}

#[post("/media/upload", data = "<upload>")]
pub async fn upload_media(
    pool: &State<MySqlPool>,
    config: &State<AppConfig>,
    user: CourseAuthorGuard,
    mut upload: Form<MediaUpload<'_>>,
) -> Result<Json<ApiResponse<MediaResponse>>, (Status, Json<ApiError>)> {
    let file = &mut upload.file;
    let file_name = file.name().unwrap_or("unnamed").to_string();
    let content_type = file.content_type()
        .map(|ct| ct.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Read file bytes from the temp file
    use std::io::Read;
    let path = file.path().ok_or_else(|| {
        (Status::BadRequest, Json(ApiError::bad_request("No file data received")))
    })?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|mut f| f.read_to_end(&mut bytes))
        .map_err(|e| (Status::InternalServerError, Json(ApiError::new(Status::InternalServerError, &format!("Failed to read upload: {}", e)))))?;

    let media = course_service::upload_media(
        pool.inner(), &config.media_upload_dir,
        &file_name, &content_type, &bytes,
        upload.lesson_id, upload.alt_text.as_deref(),
        user.claims.user_id,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;

    Ok(ApiResponse::ok(media))
}

#[post("/media", data = "<body>")]
pub async fn register_media(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    body: Json<CreateMediaRequest>,
) -> Result<Json<ApiResponse<MediaResponse>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let media = course_service::register_media(pool.inner(), &body, user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(media))
}

#[post("/media/<uuid>/validate")]
pub async fn validate_media(
    pool: &State<MySqlPool>,
    _user: CourseAuthorGuard,
    uuid: String,
) -> Result<Json<ApiResponse<MediaResponse>>, (Status, Json<ApiError>)> {
    let result = course_service::validate_media(pool.inner(), &uuid)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(result))
}

// --- Versions ---
#[get("/<course_uuid>/versions")]
pub async fn list_versions(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    course_uuid: String,
) -> Result<Json<ApiResponse<Vec<VersionResponse>>>, (Status, Json<ApiError>)> {
    let versions = course_service::list_versions(
        pool.inner(), &course_uuid, &user.claims.role, user.claims.user_id, user.claims.department_id,
    ).await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(versions))
}

pub fn routes() -> Vec<Route> {
    routes![
        create_course, list_courses, get_course, update_course, delete_course,
        create_section, list_sections, update_section, delete_section,
        create_lesson, update_lesson, delete_lesson,
        upload_media, register_media, validate_media, list_versions,
    ]
}
