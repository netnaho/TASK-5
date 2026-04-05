use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::course::*;
use crate::models::course::{Course, CourseStatus, ALLOWED_MEDIA_TYPES, MAX_MEDIA_SIZE_BYTES};
use crate::repositories::{course_repo, audit_repo, term_repo};
use crate::utils::errors::AppError;

// ---------------------------------------------------------------------------
// Authorization helpers
// ---------------------------------------------------------------------------

/// Returns true when the course's term is compatible with the currently active term.
///
/// Rules:
/// - Course has no term (`term_id IS NULL`): always in-scope (term-agnostic content).
/// - No active term is configured:           no restriction (no term filtering).
/// - Course term == active term:              in-scope.
/// - Course term != active term:             out-of-scope for scoped roles.
pub(crate) fn term_matches(course_term: Option<i64>, active_term: Option<i64>) -> bool {
    match (course_term, active_term) {
        (Some(ct), Some(at)) => ct == at,
        _ => true,
    }
}

fn can_view_course(
    course: &Course,
    role: &str,
    user_id: i64,
    department_id: Option<i64>,
    active_term_id: Option<i64>,
) -> bool {
    let in_term = term_matches(course.term_id, active_term_id);

    match role {
        "admin" => true,
        "staff_author" => {
            // Own courses are always visible regardless of term (authors manage
            // their content across all terms).
            if course.created_by == Some(user_id) {
                return true;
            }
            // Non-own visible content must be in the active term.
            course.status != "draft" && in_term
        }
        "dept_reviewer" => {
            let in_dept = department_id.map_or(false, |d| course.department_id == Some(d));
            // Reviewers see their department courses (for review) or any published
            // course — both subject to term scope.
            (in_dept || course.status == "published") && in_term
        }
        _ => course.status == "published" && in_term,
    }
}

fn assert_course_owner(course: &Course, role: &str, user_id: i64) -> Result<(), AppError> {
    if role == "admin" || course.created_by == Some(user_id) {
        Ok(())
    } else {
        Err(AppError::Forbidden("You do not own this course".to_string()))
    }
}

// ---------------------------------------------------------------------------
// Courses
// ---------------------------------------------------------------------------

pub async fn create_course(
    pool: &MySqlPool, req: &CreateCourseRequest, user_id: i64, correlation_id: Option<&str>,
) -> Result<String, AppError> {
    let uuid = Uuid::new_v4().to_string();
    course_repo::create_course(
        pool, &uuid, &req.title, &req.code, req.description.as_deref(),
        req.department_id, req.term_id, user_id, req.max_enrollment,
    ).await.map_err(|e| {
        if e.to_string().contains("Duplicate") {
            AppError::Validation(format!("Course code '{}' already exists", req.code))
        } else {
            AppError::Database(e)
        }
    })?;

    let course = course_repo::find_course_by_uuid(pool, &uuid).await?
        .ok_or_else(|| AppError::Internal("Failed to retrieve created course".to_string()))?;

    if let Some(tag_ids) = &req.tag_ids {
        course_repo::set_course_tags(pool, course.id, tag_ids).await?;
    }

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "course.create",
        "course", Some(course.id), None,
        Some(&serde_json::json!({"title": req.title, "code": req.code})),
        None, None, correlation_id,
    ).await;

    Ok(uuid)
}

pub async fn get_course(
    pool: &MySqlPool, uuid: &str, role: &str, user_id: i64, department_id: Option<i64>,
) -> Result<CourseResponse, AppError> {
    let course = course_repo::find_course_by_uuid(pool, uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    let active_term_id = term_repo::find_active_term(pool).await
        .map_err(AppError::Database)?
        .map(|t| t.id);

    if !can_view_course(&course, role, user_id, department_id, active_term_id) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let tags = course_repo::get_course_tags(pool, course.id).await?;
    Ok(build_course_response(course, tags))
}

pub async fn list_courses(
    pool: &MySqlPool, role: &str, user_id: i64, department_id: Option<i64>,
) -> Result<Vec<CourseResponse>, AppError> {
    let active_term_id = term_repo::find_active_term(pool).await
        .map_err(AppError::Database)?
        .map(|t| t.id);

    let courses = match role {
        "admin" => {
            sqlx::query_as::<_, crate::models::course::Course>(
                "SELECT * FROM courses ORDER BY updated_at DESC"
            ).fetch_all(pool).await?
        }
        "staff_author" => {
            // Authors see all their own courses (no term restriction).
            course_repo::list_courses_for_author(pool, user_id).await?
        }
        "dept_reviewer" => {
            if let Some(dept_id) = department_id {
                if let Some(term_id) = active_term_id {
                    // Active term: show dept courses in that term + unscoped courses.
                    course_repo::list_courses_by_department_scoped(pool, dept_id, term_id).await?
                } else {
                    // No active term: show all dept courses.
                    course_repo::list_courses_by_department(pool, dept_id).await?
                }
            } else {
                vec![]
            }
        }
        "faculty" | "student" => {
            if let Some(term_id) = active_term_id {
                // Active term: published courses in that term + unscoped published courses.
                course_repo::list_published_courses_scoped(pool, term_id).await?
            } else {
                // No active term: all published courses.
                course_repo::list_published_courses(pool).await?
            }
        }
        _ => vec![],
    };

    let mut responses = Vec::new();
    for course in courses {
        let tags = course_repo::get_course_tags(pool, course.id).await.unwrap_or_default();
        responses.push(build_course_response(course, tags));
    }
    Ok(responses)
}

pub async fn update_course(
    pool: &MySqlPool, uuid: &str, req: &UpdateCourseRequest, user_id: i64, role: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let course = course_repo::find_course_by_uuid(pool, uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    let status = CourseStatus::from_str(&course.status);
    match status {
        Some(CourseStatus::Draft) | Some(CourseStatus::Rejected) => {}
        _ => return Err(AppError::Validation(format!(
            "Cannot edit course in '{}' status. Only draft or rejected courses can be edited.", course.status
        ))),
    }

    assert_course_owner(&course, role, user_id)?;

    let old_values = serde_json::json!({"title": course.title, "description": course.description});

    course_repo::update_course(
        pool, course.id,
        req.title.as_deref(), req.description.as_deref(),
        req.department_id, req.term_id, req.max_enrollment,
    ).await?;

    if let Some(tag_ids) = &req.tag_ids {
        course_repo::set_course_tags(pool, course.id, tag_ids).await?;
    }

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "course.update",
        "course", Some(course.id), Some(&old_values),
        Some(&serde_json::json!({"title": req.title, "description": req.description})),
        None, None, correlation_id,
    ).await;

    Ok(())
}

pub async fn delete_course(
    pool: &MySqlPool, uuid: &str, user_id: i64, role: &str, correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let course = course_repo::find_course_by_uuid(pool, uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    assert_course_owner(&course, role, user_id)?;

    if course.status != "draft" {
        return Err(AppError::Validation("Only draft courses can be deleted".to_string()));
    }

    course_repo::delete_course(pool, course.id).await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "course.delete",
        "course", Some(course.id),
        Some(&serde_json::json!({"title": course.title, "code": course.code})),
        None, None, None, correlation_id,
    ).await;

    Ok(())
}

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

pub async fn create_section(
    pool: &MySqlPool, course_uuid: &str, req: &CreateSectionRequest, user_id: i64, role: &str,
) -> Result<String, AppError> {
    let course = course_repo::find_course_by_uuid(pool, course_uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;
    assert_course_owner(&course, role, user_id)?;
    let uuid = Uuid::new_v4().to_string();
    let sort_order = req.sort_order.unwrap_or(0);
    course_repo::create_section(pool, &uuid, course.id, &req.title, req.description.as_deref(), sort_order).await?;
    Ok(uuid)
}

pub async fn list_sections_with_lessons(
    pool: &MySqlPool, course_uuid: &str,
    role: &str, user_id: i64, department_id: Option<i64>,
) -> Result<Vec<SectionResponse>, AppError> {
    let course = course_repo::find_course_by_uuid(pool, course_uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    let active_term_id = term_repo::find_active_term(pool).await
        .map_err(AppError::Database)?
        .map(|t| t.id);

    if !can_view_course(&course, role, user_id, department_id, active_term_id) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let sections = course_repo::list_sections(pool, course.id).await?;
    let mut result = Vec::new();
    for sec in sections {
        let lessons = course_repo::list_lessons(pool, sec.id).await?;
        result.push(SectionResponse {
            uuid: sec.uuid,
            course_id: sec.course_id,
            title: sec.title,
            description: sec.description,
            sort_order: sec.sort_order,
            is_published: sec.is_published,
            lessons: lessons.into_iter().map(|l| LessonResponse {
                uuid: l.uuid, section_id: l.section_id, title: l.title,
                content_type: l.content_type, content_body: l.content_body,
                content_html: l.content_html, sort_order: l.sort_order,
                duration_minutes: l.duration_minutes, is_published: l.is_published,
            }).collect(),
        });
    }
    Ok(result)
}

pub async fn update_section(
    pool: &MySqlPool, section_uuid: &str, req: &UpdateSectionRequest,
    user_id: i64, role: &str,
) -> Result<(), AppError> {
    let section = course_repo::find_section_by_uuid(pool, section_uuid).await?
        .ok_or_else(|| AppError::NotFound("Section not found".to_string()))?;
    let course = course_repo::find_course_by_id(pool, section.course_id).await?
        .ok_or_else(|| AppError::Internal("Course not found".to_string()))?;
    assert_course_owner(&course, role, user_id)?;
    course_repo::update_section(pool, section.id, req.title.as_deref(), req.description.as_deref(), req.sort_order).await?;
    Ok(())
}

pub async fn delete_section(
    pool: &MySqlPool, section_uuid: &str, user_id: i64, role: &str,
) -> Result<(), AppError> {
    let section = course_repo::find_section_by_uuid(pool, section_uuid).await?
        .ok_or_else(|| AppError::NotFound("Section not found".to_string()))?;
    let course = course_repo::find_course_by_id(pool, section.course_id).await?
        .ok_or_else(|| AppError::Internal("Course not found".to_string()))?;
    assert_course_owner(&course, role, user_id)?;
    course_repo::delete_section(pool, section.id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Lessons
// ---------------------------------------------------------------------------

pub async fn create_lesson(
    pool: &MySqlPool, section_uuid: &str, req: &CreateLessonRequest,
    user_id: i64, role: &str,
) -> Result<String, AppError> {
    let section = course_repo::find_section_by_uuid(pool, section_uuid).await?
        .ok_or_else(|| AppError::NotFound("Section not found".to_string()))?;
    let course = course_repo::find_course_by_id(pool, section.course_id).await?
        .ok_or_else(|| AppError::Internal("Course not found".to_string()))?;
    assert_course_owner(&course, role, user_id)?;
    let uuid = Uuid::new_v4().to_string();
    let content_type = req.content_type.as_deref().unwrap_or("text");
    let sort_order = req.sort_order.unwrap_or(0);
    course_repo::create_lesson(pool, &uuid, section.id, &req.title, content_type,
        req.content_body.as_deref(), req.content_html.as_deref(), sort_order, req.duration_minutes).await?;
    Ok(uuid)
}

pub async fn update_lesson(
    pool: &MySqlPool, lesson_uuid: &str, req: &UpdateLessonRequest,
    user_id: i64, role: &str,
) -> Result<(), AppError> {
    let lesson = course_repo::find_lesson_by_uuid(pool, lesson_uuid).await?
        .ok_or_else(|| AppError::NotFound("Lesson not found".to_string()))?;
    let section = course_repo::find_section_by_id(pool, lesson.section_id).await?
        .ok_or_else(|| AppError::Internal("Section not found".to_string()))?;
    let course = course_repo::find_course_by_id(pool, section.course_id).await?
        .ok_or_else(|| AppError::Internal("Course not found".to_string()))?;
    assert_course_owner(&course, role, user_id)?;
    course_repo::update_lesson(pool, lesson.id, req.title.as_deref(), req.content_type.as_deref(),
        req.content_body.as_deref(), req.content_html.as_deref(), req.sort_order, req.duration_minutes).await?;
    Ok(())
}

pub async fn delete_lesson(
    pool: &MySqlPool, lesson_uuid: &str, user_id: i64, role: &str,
) -> Result<(), AppError> {
    let lesson = course_repo::find_lesson_by_uuid(pool, lesson_uuid).await?
        .ok_or_else(|| AppError::NotFound("Lesson not found".to_string()))?;
    let section = course_repo::find_section_by_id(pool, lesson.section_id).await?
        .ok_or_else(|| AppError::Internal("Section not found".to_string()))?;
    let course = course_repo::find_course_by_id(pool, section.course_id).await?
        .ok_or_else(|| AppError::Internal("Course not found".to_string()))?;
    assert_course_owner(&course, role, user_id)?;
    course_repo::delete_lesson(pool, lesson.id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

pub async fn register_media(pool: &MySqlPool, req: &CreateMediaRequest, user_id: i64) -> Result<MediaResponse, AppError> {
    if !ALLOWED_MEDIA_TYPES.contains(&req.mime_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid media type '{}'. Allowed: PDF, MP4, PNG", req.mime_type
        )));
    }
    if req.file_size_bytes > MAX_MEDIA_SIZE_BYTES {
        return Err(AppError::Validation(format!(
            "File size {} bytes exceeds maximum of 500 MB", req.file_size_bytes
        )));
    }

    let uuid = Uuid::new_v4().to_string();
    course_repo::create_media(pool, &uuid, req.lesson_id, user_id,
        &req.file_name, &req.file_path, &req.mime_type, req.file_size_bytes,
        req.checksum.as_deref(), req.alt_text.as_deref(), false, None).await?;

    Ok(MediaResponse {
        uuid, file_name: req.file_name.clone(), file_path: req.file_path.clone(),
        mime_type: req.mime_type.clone(), file_size_bytes: req.file_size_bytes,
        status: "pending_scan".to_string(), validated: false, validation_error: None,
    })
}

/// Upload a media file from binary data. Validates MIME type and size on the actual
/// payload, stores the file to disk, creates the media_assets record, and returns
/// the asset in pending_scan status (call validate_media next).
pub async fn upload_media(
    pool: &MySqlPool,
    upload_dir: &str,
    file_name: &str,
    content_type: &str,
    file_bytes: &[u8],
    lesson_id: Option<i64>,
    alt_text: Option<&str>,
    user_id: i64,
) -> Result<MediaResponse, AppError> {
    // Validate MIME type
    if !ALLOWED_MEDIA_TYPES.contains(&content_type) {
        return Err(AppError::Validation(format!(
            "Invalid media type '{}'. Allowed: PDF, MP4, PNG", content_type
        )));
    }

    let file_size = file_bytes.len() as i64;

    // Validate size
    if file_size > MAX_MEDIA_SIZE_BYTES {
        return Err(AppError::Validation(format!(
            "File size {} bytes exceeds maximum of 500 MB", file_size
        )));
    }

    // Generate UUID and determine storage path
    let uuid = Uuid::new_v4().to_string();
    let extension = std::path::Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let stored_name = format!("{}.{}", uuid, extension);
    let file_path = format!("{}/{}", upload_dir, stored_name);

    // Ensure upload directory exists
    std::fs::create_dir_all(upload_dir)
        .map_err(|e| AppError::Internal(format!("Cannot create upload directory: {}", e)))?;

    // Write file to disk
    std::fs::write(&file_path, file_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to write media file: {}", e)))?;

    // Compute SHA-256 checksum
    use sha2::{Sha256, Digest};
    let checksum = format!("{:x}", Sha256::digest(file_bytes));

    // Create database record
    course_repo::create_media(
        pool, &uuid, lesson_id, user_id,
        file_name, &file_path, content_type, file_size,
        Some(&checksum), alt_text, false, None,
    ).await?;

    Ok(MediaResponse {
        uuid, file_name: file_name.to_string(), file_path,
        mime_type: content_type.to_string(), file_size_bytes: file_size,
        status: "pending_scan".to_string(), validated: false, validation_error: None,
    })
}

pub async fn validate_media(pool: &MySqlPool, media_uuid: &str) -> Result<MediaResponse, AppError> {
    let media = course_repo::find_media_by_uuid(pool, media_uuid).await?
        .ok_or_else(|| AppError::NotFound("Media not found".to_string()))?;

    if media.status != "pending_scan" {
        return Err(AppError::Validation(format!(
            "Media is already in '{}' status and cannot be re-validated", media.status
        )));
    }

    // Deterministic validation checks
    let mut errors = Vec::new();

    // 1. File extension matches MIME type
    let expected_extensions: &[&str] = match media.mime_type.as_str() {
        "application/pdf" => &[".pdf"],
        "video/mp4" => &[".mp4"],
        "image/png" => &[".png"],
        _ => &[],
    };
    if !expected_extensions.iter().any(|ext| media.file_name.to_lowercase().ends_with(ext)) {
        errors.push(format!("File extension does not match MIME type {}", media.mime_type));
    }

    // 2. Checksum must be present
    if media.checksum.as_ref().map_or(true, |c| c.is_empty()) {
        errors.push("Checksum is missing".to_string());
    }

    // 3. File path must be absolute
    if !media.file_path.starts_with('/') {
        errors.push("File path must be absolute".to_string());
    }

    if errors.is_empty() {
        course_repo::update_media_status(pool, media.id, "ready", true, None).await?;
        Ok(MediaResponse {
            uuid: media.uuid, file_name: media.file_name, file_path: media.file_path,
            mime_type: media.mime_type, file_size_bytes: media.file_size_bytes,
            status: "ready".to_string(), validated: true, validation_error: None,
        })
    } else {
        let error_msg = errors.join("; ");
        course_repo::update_media_status(pool, media.id, "failed", false, Some(&error_msg)).await?;
        Ok(MediaResponse {
            uuid: media.uuid, file_name: media.file_name, file_path: media.file_path,
            mime_type: media.mime_type, file_size_bytes: media.file_size_bytes,
            status: "failed".to_string(), validated: false, validation_error: Some(error_msg),
        })
    }
}

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

pub async fn create_tag(pool: &MySqlPool, name: &str) -> Result<TagResponse, AppError> {
    let uuid = Uuid::new_v4().to_string();
    let slug = name.to_lowercase().replace(' ', "-").replace(|c: char| !c.is_alphanumeric() && c != '-', "");
    if let Some(existing) = course_repo::find_tag_by_slug(pool, &slug).await? {
        return Ok(TagResponse { id: existing.id, uuid: existing.uuid, name: existing.name, slug: existing.slug });
    }
    let id = course_repo::create_tag(pool, &uuid, name, &slug).await
        .map_err(|e| {
            if e.to_string().contains("Duplicate") {
                AppError::Validation(format!("Tag '{}' already exists", name))
            } else {
                AppError::Database(e)
            }
        })?;
    Ok(TagResponse { id: id as i64, uuid, name: name.to_string(), slug })
}

pub async fn list_tags(pool: &MySqlPool) -> Result<Vec<TagResponse>, AppError> {
    let tags = course_repo::list_tags(pool).await?;
    Ok(tags.into_iter().map(|t| TagResponse { id: t.id, uuid: t.uuid, name: t.name, slug: t.slug }).collect())
}

// ---------------------------------------------------------------------------
// Versions
// ---------------------------------------------------------------------------

pub async fn list_versions(
    pool: &MySqlPool, course_uuid: &str,
    role: &str, user_id: i64, department_id: Option<i64>,
) -> Result<Vec<VersionResponse>, AppError> {
    let course = course_repo::find_course_by_uuid(pool, course_uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    let active_term_id = term_repo::find_active_term(pool).await
        .map_err(AppError::Database)?
        .map(|t| t.id);

    if !can_view_course(&course, role, user_id, department_id, active_term_id) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let versions = course_repo::list_versions(pool, course.id).await
        .map_err(AppError::Database)?;
    Ok(versions.into_iter().map(|v| VersionResponse {
        uuid: v.uuid,
        version_number: v.version_number,
        change_summary: v.change_summary,
        snapshot: v.snapshot,
        created_at: v.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        expires_at: v.expires_at.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
    }).collect())
}

fn build_course_response(course: crate::models::course::Course, tags: Vec<crate::models::course::Tag>) -> CourseResponse {
    CourseResponse {
        uuid: course.uuid,
        title: course.title,
        code: course.code,
        description: course.description,
        department_id: course.department_id,
        term_id: course.term_id,
        instructor_id: course.instructor_id,
        status: course.status,
        visibility: course.visibility,
        max_enrollment: course.max_enrollment,
        current_version: course.current_version,
        release_notes: course.release_notes,
        effective_date: course.effective_date.map(|d| d.format("%m/%d/%Y %I:%M %p").to_string()),
        updated_on: course.updated_on.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
        tags: tags.into_iter().map(|t| TagResponse { id: t.id, uuid: t.uuid, name: t.name, slug: t.slug }).collect(),
        created_at: course.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        updated_at: course.updated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }
}
