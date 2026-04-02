use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::course::*;
use crate::models::course::{CourseStatus, ALLOWED_MEDIA_TYPES, MAX_MEDIA_SIZE_BYTES};
use crate::repositories::{course_repo, audit_repo};
use crate::utils::errors::AppError;

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

    // Get the created course to get its ID
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

pub async fn get_course(pool: &MySqlPool, uuid: &str) -> Result<CourseResponse, AppError> {
    let course = course_repo::find_course_by_uuid(pool, uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;
    let tags = course_repo::get_course_tags(pool, course.id).await?;
    Ok(build_course_response(course, tags))
}

pub async fn list_courses(
    pool: &MySqlPool, role: &str, user_id: i64, department_id: Option<i64>,
) -> Result<Vec<CourseResponse>, AppError> {
    let courses = match role {
        "admin" => {
            // Admin sees all courses - get from all departments
            // We'll just get all courses with a simple query
            sqlx::query_as::<_, crate::models::course::Course>(
                "SELECT * FROM courses ORDER BY updated_at DESC"
            ).fetch_all(pool).await?
        }
        "staff_author" => course_repo::list_courses_for_author(pool, user_id).await?,
        "dept_reviewer" => {
            if let Some(dept_id) = department_id {
                course_repo::list_courses_by_department(pool, dept_id).await?
            } else {
                vec![]
            }
        }
        "faculty" | "student" => course_repo::list_published_courses(pool).await?,
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
    pool: &MySqlPool, uuid: &str, req: &UpdateCourseRequest, user_id: i64, correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let course = course_repo::find_course_by_uuid(pool, uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    // Only draft or rejected courses can be edited
    let status = CourseStatus::from_str(&course.status);
    match status {
        Some(CourseStatus::Draft) | Some(CourseStatus::Rejected) => {}
        _ => return Err(AppError::Validation(format!(
            "Cannot edit course in '{}' status. Only draft or rejected courses can be edited.", course.status
        ))),
    }

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
    pool: &MySqlPool, uuid: &str, user_id: i64, correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let course = course_repo::find_course_by_uuid(pool, uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

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

// --- Sections ---
pub async fn create_section(
    pool: &MySqlPool, course_uuid: &str, req: &CreateSectionRequest, user_id: i64,
) -> Result<String, AppError> {
    let course = course_repo::find_course_by_uuid(pool, course_uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;
    let uuid = Uuid::new_v4().to_string();
    let sort_order = req.sort_order.unwrap_or(0);
    course_repo::create_section(pool, &uuid, course.id, &req.title, req.description.as_deref(), sort_order).await?;
    Ok(uuid)
}

pub async fn list_sections_with_lessons(pool: &MySqlPool, course_uuid: &str) -> Result<Vec<SectionResponse>, AppError> {
    let course = course_repo::find_course_by_uuid(pool, course_uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;
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

pub async fn update_section(pool: &MySqlPool, section_uuid: &str, req: &UpdateSectionRequest) -> Result<(), AppError> {
    let section = course_repo::find_section_by_uuid(pool, section_uuid).await?
        .ok_or_else(|| AppError::NotFound("Section not found".to_string()))?;
    course_repo::update_section(pool, section.id, req.title.as_deref(), req.description.as_deref(), req.sort_order).await?;
    Ok(())
}

pub async fn delete_section(pool: &MySqlPool, section_uuid: &str) -> Result<(), AppError> {
    let section = course_repo::find_section_by_uuid(pool, section_uuid).await?
        .ok_or_else(|| AppError::NotFound("Section not found".to_string()))?;
    course_repo::delete_section(pool, section.id).await?;
    Ok(())
}

// --- Lessons ---
pub async fn create_lesson(pool: &MySqlPool, section_uuid: &str, req: &CreateLessonRequest) -> Result<String, AppError> {
    let section = course_repo::find_section_by_uuid(pool, section_uuid).await?
        .ok_or_else(|| AppError::NotFound("Section not found".to_string()))?;
    let uuid = Uuid::new_v4().to_string();
    let content_type = req.content_type.as_deref().unwrap_or("text");
    let sort_order = req.sort_order.unwrap_or(0);
    course_repo::create_lesson(pool, &uuid, section.id, &req.title, content_type,
        req.content_body.as_deref(), req.content_html.as_deref(), sort_order, req.duration_minutes).await?;
    Ok(uuid)
}

pub async fn update_lesson(pool: &MySqlPool, lesson_uuid: &str, req: &UpdateLessonRequest) -> Result<(), AppError> {
    let lesson = course_repo::find_lesson_by_uuid(pool, lesson_uuid).await?
        .ok_or_else(|| AppError::NotFound("Lesson not found".to_string()))?;
    course_repo::update_lesson(pool, lesson.id, req.title.as_deref(), req.content_type.as_deref(),
        req.content_body.as_deref(), req.content_html.as_deref(), req.sort_order, req.duration_minutes).await?;
    Ok(())
}

pub async fn delete_lesson(pool: &MySqlPool, lesson_uuid: &str) -> Result<(), AppError> {
    let lesson = course_repo::find_lesson_by_uuid(pool, lesson_uuid).await?
        .ok_or_else(|| AppError::NotFound("Lesson not found".to_string()))?;
    course_repo::delete_lesson(pool, lesson.id).await?;
    Ok(())
}

// --- Media ---
pub async fn register_media(pool: &MySqlPool, req: &CreateMediaRequest, user_id: i64) -> Result<MediaResponse, AppError> {
    // Validate media type
    if !ALLOWED_MEDIA_TYPES.contains(&req.mime_type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid media type '{}'. Allowed: PDF, MP4, PNG", req.mime_type
        )));
    }
    // Validate size
    if req.file_size_bytes > MAX_MEDIA_SIZE_BYTES {
        return Err(AppError::Validation(format!(
            "File size {} bytes exceeds maximum of 500 MB", req.file_size_bytes
        )));
    }

    let uuid = Uuid::new_v4().to_string();
    let validated = true;
    course_repo::create_media(pool, &uuid, req.lesson_id, user_id,
        &req.file_name, &req.file_path, &req.mime_type, req.file_size_bytes,
        req.checksum.as_deref(), req.alt_text.as_deref(), validated, None).await?;

    Ok(MediaResponse {
        uuid, file_name: req.file_name.clone(), file_path: req.file_path.clone(),
        mime_type: req.mime_type.clone(), file_size_bytes: req.file_size_bytes,
        status: "ready".to_string(), validated: true, validation_error: None,
    })
}

// --- Tags ---
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
