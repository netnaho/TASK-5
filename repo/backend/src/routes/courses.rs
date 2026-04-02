use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;
use rocket::State;
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
    _user: AuthenticatedUser,
    uuid: String,
) -> Result<Json<ApiResponse<CourseResponse>>, (Status, Json<ApiError>)> {
    let course = course_service::get_course(pool.inner(), &uuid)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(course))
}

#[put("/<uuid>", data = "<body>")]
pub async fn update_course(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
    body: Json<UpdateCourseRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::update_course(pool.inner(), &uuid, &body, user.claims.user_id, None)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Course updated".to_string()))
}

#[delete("/<uuid>")]
pub async fn delete_course(
    pool: &State<MySqlPool>,
    user: CourseAuthorGuard,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::delete_course(pool.inner(), &uuid, user.claims.user_id, None)
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
    let uuid = course_service::create_section(pool.inner(), &course_uuid, &body, user.claims.user_id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

#[get("/<course_uuid>/sections")]
pub async fn list_sections(
    pool: &State<MySqlPool>,
    _user: AuthenticatedUser,
    course_uuid: String,
) -> Result<Json<ApiResponse<Vec<SectionResponse>>>, (Status, Json<ApiError>)> {
    let sections = course_service::list_sections_with_lessons(pool.inner(), &course_uuid)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(sections))
}

#[put("/sections/<uuid>", data = "<body>")]
pub async fn update_section(
    pool: &State<MySqlPool>,
    _user: CourseAuthorGuard,
    uuid: String,
    body: Json<UpdateSectionRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::update_section(pool.inner(), &uuid, &body)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Section updated".to_string()))
}

#[delete("/sections/<uuid>")]
pub async fn delete_section(
    pool: &State<MySqlPool>,
    _user: CourseAuthorGuard,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::delete_section(pool.inner(), &uuid)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Section deleted".to_string()))
}

// --- Lessons ---
#[post("/sections/<section_uuid>/lessons", data = "<body>")]
pub async fn create_lesson(
    pool: &State<MySqlPool>,
    _user: CourseAuthorGuard,
    section_uuid: String,
    body: Json<CreateLessonRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (Status, Json<ApiError>)> {
    if let Err(e) = body.validate() {
        let msg = e.field_errors().values()
            .flat_map(|errs| errs.iter().filter_map(|e| e.message.as_ref().map(|m| m.to_string())))
            .collect::<Vec<_>>().join("; ");
        return Err((Status::BadRequest, Json(ApiError::bad_request(msg))));
    }
    let uuid = course_service::create_lesson(pool.inner(), &section_uuid, &body)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok(serde_json::json!({"uuid": uuid})))
}

#[put("/lessons/<uuid>", data = "<body>")]
pub async fn update_lesson(
    pool: &State<MySqlPool>,
    _user: CourseAuthorGuard,
    uuid: String,
    body: Json<UpdateLessonRequest>,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::update_lesson(pool.inner(), &uuid, &body)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Lesson updated".to_string()))
}

#[delete("/lessons/<uuid>")]
pub async fn delete_lesson(
    pool: &State<MySqlPool>,
    _user: CourseAuthorGuard,
    uuid: String,
) -> Result<Json<ApiResponse<String>>, (Status, Json<ApiError>)> {
    course_service::delete_lesson(pool.inner(), &uuid)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(e))?;
    Ok(ApiResponse::ok("Lesson deleted".to_string()))
}

// --- Media ---
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

// --- Versions ---
#[get("/<course_uuid>/versions")]
pub async fn list_versions(
    pool: &State<MySqlPool>,
    _user: AuthenticatedUser,
    course_uuid: String,
) -> Result<Json<ApiResponse<Vec<crate::dto::course::VersionResponse>>>, (Status, Json<ApiError>)> {
    let course = crate::repositories::course_repo::find_course_by_uuid(pool.inner(), &course_uuid)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(crate::utils::errors::AppError::Database(e)))?
        .ok_or_else(|| (Status::NotFound, Json(ApiError::not_found("Course not found"))))?;

    let versions = crate::repositories::course_repo::list_versions(pool.inner(), course.id)
        .await.map_err(|e| <(Status, Json<ApiError>)>::from(crate::utils::errors::AppError::Database(e)))?;

    let result: Vec<VersionResponse> = versions.into_iter().map(|v| VersionResponse {
        uuid: v.uuid,
        version_number: v.version_number,
        change_summary: v.change_summary,
        snapshot: v.snapshot,
        created_at: v.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        expires_at: v.expires_at.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
    }).collect();

    Ok(ApiResponse::ok(result))
}

pub fn routes() -> Vec<Route> {
    routes![
        create_course, list_courses, get_course, update_course, delete_course,
        create_section, list_sections, update_section, delete_section,
        create_lesson, update_lesson, delete_lesson,
        register_media, list_versions,
    ]
}
