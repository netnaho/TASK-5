use sqlx::MySqlPool;
use crate::models::course::{Course, CourseSection, Lesson, MediaAsset, Tag, CourseVersion, VersionDiff};

pub async fn create_course(
    pool: &MySqlPool, uuid: &str, title: &str, code: &str, description: Option<&str>,
    department_id: Option<i64>, term_id: Option<i64>, created_by: i64,
    max_enrollment: Option<i32>,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO courses (uuid, title, code, description, department_id, term_id, created_by, max_enrollment, status, visibility) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'draft', 'private')"
    )
    .bind(uuid).bind(title).bind(code).bind(description)
    .bind(department_id).bind(term_id).bind(created_by).bind(max_enrollment)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn find_course_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn find_course_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE id = ?")
        .bind(id).fetch_optional(pool).await
}

pub async fn list_courses_for_author(pool: &MySqlPool, user_id: i64) -> Result<Vec<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE created_by = ? ORDER BY updated_at DESC")
        .bind(user_id).fetch_all(pool).await
}

pub async fn list_courses_by_department(pool: &MySqlPool, dept_id: i64) -> Result<Vec<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE department_id = ? ORDER BY updated_at DESC")
        .bind(dept_id).fetch_all(pool).await
}

pub async fn list_published_courses(pool: &MySqlPool) -> Result<Vec<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE status = 'published' ORDER BY title")
        .fetch_all(pool).await
}

pub async fn list_courses_by_department_and_term(pool: &MySqlPool, dept_id: i64, term_id: i64) -> Result<Vec<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE department_id = ? AND term_id = ? ORDER BY title")
        .bind(dept_id).bind(term_id).fetch_all(pool).await
}

/// Courses in a department for the active term, plus unscoped courses (term_id IS NULL).
/// Used for dept_reviewer listings when an active term is configured.
pub async fn list_courses_by_department_scoped(pool: &MySqlPool, dept_id: i64, term_id: i64) -> Result<Vec<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "SELECT * FROM courses WHERE department_id = ? AND (term_id = ? OR term_id IS NULL) ORDER BY updated_at DESC"
    ).bind(dept_id).bind(term_id).fetch_all(pool).await
}

/// Published courses for the active term, plus unscoped published courses (term_id IS NULL).
/// Used for faculty/student listings when an active term is configured.
pub async fn list_published_courses_scoped(pool: &MySqlPool, term_id: i64) -> Result<Vec<Course>, sqlx::Error> {
    sqlx::query_as::<_, Course>(
        "SELECT * FROM courses WHERE status = 'published' AND (term_id = ? OR term_id IS NULL) ORDER BY title"
    ).bind(term_id).fetch_all(pool).await
}

pub async fn update_course(
    pool: &MySqlPool, id: i64, title: Option<&str>, description: Option<&str>,
    department_id: Option<i64>, term_id: Option<i64>, max_enrollment: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE courses SET title = COALESCE(?, title), description = COALESCE(?, description), department_id = COALESCE(?, department_id), term_id = COALESCE(?, term_id), max_enrollment = COALESCE(?, max_enrollment), updated_at = NOW(), updated_on = NOW() WHERE id = ?")
        .bind(title).bind(description).bind(department_id).bind(term_id).bind(max_enrollment).bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn update_course_status(pool: &MySqlPool, id: i64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE courses SET status = ?, updated_at = NOW(), updated_on = NOW() WHERE id = ?")
        .bind(status).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn update_course_version_info(
    pool: &MySqlPool, id: i64, version: i32, release_notes: Option<&str>, effective_date: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE courses SET current_version = ?, release_notes = ?, effective_date = ?, updated_at = NOW() WHERE id = ?")
        .bind(version).bind(release_notes).bind(effective_date).bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn delete_course(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM courses WHERE id = ? AND status = 'draft'")
        .bind(id).execute(pool).await?;
    Ok(())
}

// --- Sections ---
pub async fn create_section(pool: &MySqlPool, uuid: &str, course_id: i64, title: &str, description: Option<&str>, sort_order: i32) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO course_sections (uuid, course_id, title, description, sort_order) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid).bind(course_id).bind(title).bind(description).bind(sort_order)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn find_section_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<CourseSection>, sqlx::Error> {
    sqlx::query_as::<_, CourseSection>("SELECT * FROM course_sections WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn find_section_by_id(pool: &MySqlPool, id: i64) -> Result<Option<CourseSection>, sqlx::Error> {
    sqlx::query_as::<_, CourseSection>("SELECT * FROM course_sections WHERE id = ?")
        .bind(id).fetch_optional(pool).await
}

pub async fn list_sections(pool: &MySqlPool, course_id: i64) -> Result<Vec<CourseSection>, sqlx::Error> {
    sqlx::query_as::<_, CourseSection>("SELECT * FROM course_sections WHERE course_id = ? ORDER BY sort_order")
        .bind(course_id).fetch_all(pool).await
}

pub async fn update_section(pool: &MySqlPool, id: i64, title: Option<&str>, description: Option<&str>, sort_order: Option<i32>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE course_sections SET title = COALESCE(?, title), description = COALESCE(?, description), sort_order = COALESCE(?, sort_order), updated_at = NOW() WHERE id = ?")
        .bind(title).bind(description).bind(sort_order).bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn delete_section(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM course_sections WHERE id = ?").bind(id).execute(pool).await?;
    Ok(())
}

// --- Lessons ---
pub async fn create_lesson(pool: &MySqlPool, uuid: &str, section_id: i64, title: &str, content_type: &str, content_body: Option<&str>, content_html: Option<&str>, sort_order: i32, duration_minutes: Option<i32>) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO lessons (uuid, section_id, title, content_type, content_body, content_html, sort_order, duration_minutes) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(uuid).bind(section_id).bind(title).bind(content_type).bind(content_body).bind(content_html).bind(sort_order).bind(duration_minutes)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn find_lesson_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<Lesson>, sqlx::Error> {
    sqlx::query_as::<_, Lesson>("SELECT * FROM lessons WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn list_lessons(pool: &MySqlPool, section_id: i64) -> Result<Vec<Lesson>, sqlx::Error> {
    sqlx::query_as::<_, Lesson>("SELECT * FROM lessons WHERE section_id = ? ORDER BY sort_order")
        .bind(section_id).fetch_all(pool).await
}

pub async fn update_lesson(pool: &MySqlPool, id: i64, title: Option<&str>, content_type: Option<&str>, content_body: Option<&str>, content_html: Option<&str>, sort_order: Option<i32>, duration_minutes: Option<i32>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE lessons SET title = COALESCE(?, title), content_type = COALESCE(?, content_type), content_body = COALESCE(?, content_body), content_html = COALESCE(?, content_html), sort_order = COALESCE(?, sort_order), duration_minutes = COALESCE(?, duration_minutes), updated_at = NOW() WHERE id = ?")
        .bind(title).bind(content_type).bind(content_body).bind(content_html).bind(sort_order).bind(duration_minutes).bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn delete_lesson(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM lessons WHERE id = ?").bind(id).execute(pool).await?;
    Ok(())
}

// --- Media ---
pub async fn create_media(pool: &MySqlPool, uuid: &str, lesson_id: Option<i64>, uploaded_by: i64, file_name: &str, file_path: &str, mime_type: &str, file_size_bytes: i64, checksum: Option<&str>, alt_text: Option<&str>, validated: bool, validation_error: Option<&str>) -> Result<u64, sqlx::Error> {
    let status = if validation_error.is_some() {
        "failed"
    } else if validated {
        "ready"
    } else {
        "pending_scan"
    };
    let r = sqlx::query("INSERT INTO media_assets (uuid, lesson_id, uploaded_by, file_name, file_path, mime_type, file_size_bytes, checksum, alt_text, status, validated, validation_error) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(uuid).bind(lesson_id).bind(uploaded_by).bind(file_name).bind(file_path).bind(mime_type).bind(file_size_bytes).bind(checksum).bind(alt_text).bind(status).bind(validated).bind(validation_error)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn update_media_status(pool: &MySqlPool, id: i64, status: &str, validated: bool, validation_error: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE media_assets SET status = ?, validated = ?, validation_error = ?, updated_at = NOW() WHERE id = ?")
        .bind(status).bind(validated).bind(validation_error).bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn count_unvalidated_media_for_course(pool: &MySqlPool, course_id: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM media_assets ma JOIN lessons l ON ma.lesson_id = l.id JOIN course_sections cs ON l.section_id = cs.id WHERE cs.course_id = ? AND ma.status != 'ready'"
    ).bind(course_id).fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn find_media_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<MediaAsset>, sqlx::Error> {
    sqlx::query_as::<_, MediaAsset>("SELECT * FROM media_assets WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn list_media_for_lesson(pool: &MySqlPool, lesson_id: i64) -> Result<Vec<MediaAsset>, sqlx::Error> {
    sqlx::query_as::<_, MediaAsset>("SELECT * FROM media_assets WHERE lesson_id = ? ORDER BY created_at")
        .bind(lesson_id).fetch_all(pool).await
}

// --- Tags ---
pub async fn create_tag(pool: &MySqlPool, uuid: &str, name: &str, slug: &str) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO tags (uuid, name, slug) VALUES (?, ?, ?)")
        .bind(uuid).bind(name).bind(slug)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn find_tag_by_slug(pool: &MySqlPool, slug: &str) -> Result<Option<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE slug = ?")
        .bind(slug).fetch_optional(pool).await
}

pub async fn list_tags(pool: &MySqlPool) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY name")
        .fetch_all(pool).await
}

pub async fn set_course_tags(pool: &MySqlPool, course_id: i64, tag_ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM course_tags WHERE course_id = ?")
        .bind(course_id).execute(pool).await?;
    for tag_id in tag_ids {
        sqlx::query("INSERT INTO course_tags (course_id, tag_id) VALUES (?, ?)")
            .bind(course_id).bind(tag_id).execute(pool).await?;
    }
    Ok(())
}

pub async fn get_course_tags(pool: &MySqlPool, course_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as::<_, Tag>("SELECT t.* FROM tags t INNER JOIN course_tags ct ON ct.tag_id = t.id WHERE ct.course_id = ? ORDER BY t.name")
        .bind(course_id).fetch_all(pool).await
}

// --- Versions ---
pub async fn create_version(pool: &MySqlPool, uuid: &str, course_id: i64, version_number: i32, snapshot: &serde_json::Value, created_by: i64, change_summary: Option<&str>, expires_at: Option<&str>) -> Result<u64, sqlx::Error> {
    let snapshot_str = serde_json::to_string(snapshot).unwrap_or_default();
    let r = sqlx::query("INSERT INTO course_versions (uuid, course_id, version_number, snapshot, created_by, change_summary, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(uuid).bind(course_id).bind(version_number).bind(snapshot_str).bind(created_by).bind(change_summary).bind(expires_at)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn get_latest_version(pool: &MySqlPool, course_id: i64) -> Result<Option<CourseVersion>, sqlx::Error> {
    sqlx::query_as::<_, CourseVersion>("SELECT * FROM course_versions WHERE course_id = ? ORDER BY version_number DESC LIMIT 1")
        .bind(course_id).fetch_optional(pool).await
}

pub async fn list_versions(pool: &MySqlPool, course_id: i64) -> Result<Vec<CourseVersion>, sqlx::Error> {
    sqlx::query_as::<_, CourseVersion>("SELECT * FROM course_versions WHERE course_id = ? ORDER BY version_number DESC")
        .bind(course_id).fetch_all(pool).await
}

pub async fn create_diff(pool: &MySqlPool, uuid: &str, course_id: i64, from_version: i32, to_version: i32, diff_data: &serde_json::Value) -> Result<u64, sqlx::Error> {
    let diff_str = serde_json::to_string(diff_data).unwrap_or_default();
    let r = sqlx::query("INSERT INTO version_diffs (uuid, course_id, from_version, to_version, diff_data) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid).bind(course_id).bind(from_version).bind(to_version).bind(diff_str)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn get_diff(pool: &MySqlPool, course_id: i64, from_v: i32, to_v: i32) -> Result<Option<VersionDiff>, sqlx::Error> {
    sqlx::query_as::<_, VersionDiff>("SELECT * FROM version_diffs WHERE course_id = ? AND from_version = ? AND to_version = ?")
        .bind(course_id).bind(from_v).bind(to_v).fetch_optional(pool).await
}

// --- Course snapshot builder ---
pub async fn build_course_snapshot(pool: &MySqlPool, course_id: i64) -> Result<serde_json::Value, sqlx::Error> {
    let course = find_course_by_id(pool, course_id).await?.unwrap();
    let sections = list_sections(pool, course_id).await?;
    let mut section_data = Vec::new();
    for sec in &sections {
        let lessons = list_lessons(pool, sec.id).await?;
        section_data.push(serde_json::json!({
            "uuid": sec.uuid,
            "title": sec.title,
            "description": sec.description,
            "sort_order": sec.sort_order,
            "lessons": lessons.iter().map(|l| serde_json::json!({
                "uuid": l.uuid,
                "title": l.title,
                "content_type": l.content_type,
                "content_body": l.content_body,
                "sort_order": l.sort_order,
                "duration_minutes": l.duration_minutes,
            })).collect::<Vec<_>>(),
        }));
    }
    let tags = get_course_tags(pool, course_id).await?;
    Ok(serde_json::json!({
        "title": course.title,
        "code": course.code,
        "description": course.description,
        "department_id": course.department_id,
        "term_id": course.term_id,
        "max_enrollment": course.max_enrollment,
        "sections": section_data,
        "tags": tags.iter().map(|t| &t.name).collect::<Vec<_>>(),
    }))
}
