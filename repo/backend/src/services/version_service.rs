use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::config::AppConfig;
use crate::repositories::course_repo;
use crate::utils::errors::AppError;

pub async fn create_version_snapshot(
    pool: &MySqlPool, config: &AppConfig, course_id: i64, user_id: i64, change_summary: Option<&str>,
) -> Result<i32, AppError> {
    let latest = course_repo::get_latest_version(pool, course_id).await?;
    let new_version = latest.as_ref().map(|v| v.version_number + 1).unwrap_or(1);

    let snapshot = course_repo::build_course_snapshot(pool, course_id).await?;

    let expires_at = (Utc::now() + Duration::days(config.version_retention_days)).format("%Y-%m-%d %H:%M:%S").to_string();

    course_repo::create_version(
        pool, &Uuid::new_v4().to_string(), course_id, new_version,
        &snapshot, user_id, change_summary, Some(&expires_at),
    ).await?;

    // Generate diff from previous version
    if let Some(prev) = latest {
        let diff = generate_diff(&prev.snapshot, &snapshot);
        course_repo::create_diff(
            pool, &Uuid::new_v4().to_string(), course_id,
            prev.version_number, new_version, &diff,
        ).await?;
    }

    // Update course current_version
    course_repo::update_course_version_info(pool, course_id, new_version, None, None).await?;

    Ok(new_version)
}

pub fn generate_diff(old: &serde_json::Value, new: &serde_json::Value) -> serde_json::Value {
    let mut changes = Vec::new();

    // Compare top-level fields
    if let (Some(old_obj), Some(new_obj)) = (old.as_object(), new.as_object()) {
        for key in new_obj.keys() {
            if key == "sections" || key == "tags" {
                continue; // Handle separately
            }
            let old_val = old_obj.get(key);
            let new_val = new_obj.get(key);
            if old_val != new_val {
                changes.push(serde_json::json!({
                    "field": key,
                    "type": "field_changed",
                    "old": old_val,
                    "new": new_val,
                }));
            }
        }

        // Compare tags
        let old_tags = old_obj.get("tags").and_then(|v| v.as_array());
        let new_tags = new_obj.get("tags").and_then(|v| v.as_array());
        if old_tags != new_tags {
            changes.push(serde_json::json!({
                "field": "tags",
                "type": "tags_changed",
                "old": old_tags,
                "new": new_tags,
            }));
        }

        // Compare sections
        let old_sections = old_obj.get("sections").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let new_sections = new_obj.get("sections").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let old_uuids: Vec<&str> = old_sections.iter().filter_map(|s| s.get("uuid").and_then(|u| u.as_str())).collect();
        let new_uuids: Vec<&str> = new_sections.iter().filter_map(|s| s.get("uuid").and_then(|u| u.as_str())).collect();

        // Added sections
        for sec in &new_sections {
            if let Some(uuid) = sec.get("uuid").and_then(|u| u.as_str()) {
                if !old_uuids.contains(&uuid) {
                    changes.push(serde_json::json!({
                        "type": "section_added",
                        "section_uuid": uuid,
                        "title": sec.get("title"),
                    }));
                }
            }
        }

        // Removed sections
        for sec in &old_sections {
            if let Some(uuid) = sec.get("uuid").and_then(|u| u.as_str()) {
                if !new_uuids.contains(&uuid) {
                    changes.push(serde_json::json!({
                        "type": "section_removed",
                        "section_uuid": uuid,
                        "title": sec.get("title"),
                    }));
                }
            }
        }

        // Modified sections (compare titles and lesson counts)
        for new_sec in &new_sections {
            if let Some(uuid) = new_sec.get("uuid").and_then(|u| u.as_str()) {
                if let Some(old_sec) = old_sections.iter().find(|s| s.get("uuid").and_then(|u| u.as_str()) == Some(uuid)) {
                    if old_sec.get("title") != new_sec.get("title") {
                        changes.push(serde_json::json!({
                            "type": "section_title_changed",
                            "section_uuid": uuid,
                            "old_title": old_sec.get("title"),
                            "new_title": new_sec.get("title"),
                        }));
                    }
                    // Compare lessons within section
                    let old_lessons = old_sec.get("lessons").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                    let new_lessons = new_sec.get("lessons").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                    if old_lessons != new_lessons {
                        changes.push(serde_json::json!({
                            "type": "section_lessons_changed",
                            "section_uuid": uuid,
                            "old_lesson_count": old_lessons,
                            "new_lesson_count": new_lessons,
                        }));
                    }
                }
            }
        }
    }

    serde_json::json!({
        "changes": changes,
        "total_changes": changes.len(),
    })
}
