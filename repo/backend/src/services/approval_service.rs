use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::config::AppConfig;
use crate::dto::approval::*;
use crate::repositories::{approval_repo, course_repo, audit_repo, security_repo, term_repo};
use crate::services::{version_service, course_service::term_matches, notification_service, term_service};
use crate::utils::errors::AppError;

pub async fn submit_for_approval(
    pool: &MySqlPool, config: &AppConfig, course_uuid: &str,
    req: &SubmitApprovalRequest, user_id: i64, role: &str, correlation_id: Option<&str>,
) -> Result<String, AppError> {
    let course = course_repo::find_course_by_uuid(pool, course_uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    // Only the course owner (or admin) may submit for approval
    if role != "admin" && course.created_by != Some(user_id) {
        return Err(AppError::Forbidden("You do not own this course".to_string()));
    }

    // Only draft or rejected courses can be submitted
    if course.status != "draft" && course.status != "rejected" {
        return Err(AppError::Validation(format!(
            "Cannot submit course in '{}' status for approval", course.status
        )));
    }

    // Enforce active term acceptance
    term_service::check_active_term_accepted(pool, user_id).await?;

    // Check all media assets are validated
    let unvalidated = course_repo::count_unvalidated_media_for_course(pool, course.id).await?;
    if unvalidated > 0 {
        return Err(AppError::Validation(format!(
            "Cannot submit for approval: {} media asset(s) are not validated. Please validate all media first.", unvalidated
        )));
    }

    // Check no active approval exists
    if let Some(_) = approval_repo::find_active_approval_for_course(pool, course.id).await? {
        return Err(AppError::Validation("An active approval request already exists for this course".to_string()));
    }

    // Parse effective date (MM/DD/YYYY HH:MM AM/PM)
    let effective_dt = parse_effective_date(&req.effective_date)?;
    let effective_str = effective_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    // Create version snapshot
    let version_num = version_service::create_version_snapshot(
        pool, config, course.id, user_id, Some(&format!("Submitted for approval: {}", req.release_notes)),
    ).await?;

    // Create approval request
    let approval_uuid = Uuid::new_v4().to_string();
    let approval_id = approval_repo::create_approval(
        pool, &approval_uuid, "course", course.id, user_id,
        &req.release_notes, &effective_str, version_num, req.notes.as_deref(),
    ).await?;

    // Create two approval steps
    approval_repo::create_step(pool, &Uuid::new_v4().to_string(), approval_id as i64, 1, "dept_reviewer").await?;
    approval_repo::create_step(pool, &Uuid::new_v4().to_string(), approval_id as i64, 2, "admin").await?;

    // Update course status
    course_repo::update_course_status(pool, course.id, "pending_approval").await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "approval.submit",
        "course", Some(course.id), None,
        Some(&serde_json::json!({"release_notes": req.release_notes, "effective_date": req.effective_date, "version": version_num})),
        None, None, correlation_id,
    ).await;

    let _ = notification_service::notify_department_role(
        pool, course.department_id, "dept_reviewer",
        "New Course Awaiting Review",
        &format!("Course '{}' has been submitted for approval.", course.title),
        "approval", Some("course"), Some(course_uuid),
    ).await;

    Ok(approval_uuid)
}

pub async fn submit_for_unpublish(
    pool: &MySqlPool, course_uuid: &str,
    req: &SubmitApprovalRequest, user_id: i64, role: &str, correlation_id: Option<&str>,
) -> Result<String, AppError> {
    let course = course_repo::find_course_by_uuid(pool, course_uuid).await?
        .ok_or_else(|| AppError::NotFound("Course not found".to_string()))?;

    // Only the course owner (or admin) may request unpublish
    if role != "admin" && course.created_by != Some(user_id) {
        return Err(AppError::Forbidden("You do not own this course".to_string()));
    }

    // Only published courses can be submitted for unpublish
    if course.status != "published" {
        return Err(AppError::Validation(format!(
            "Cannot submit course in '{}' status for unpublish; only published courses may be unpublished", course.status
        )));
    }

    // Check no active approval (publish or unpublish) exists
    if let Some(_) = approval_repo::find_active_approval_for_course(pool, course.id).await? {
        return Err(AppError::Validation("An active approval request already exists for this course".to_string()));
    }

    // Parse effective date (same rules as publish)
    let effective_dt = parse_effective_date(&req.effective_date)?;
    let effective_str = effective_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    // Create approval request with entity_type = 'course_unpublish' (no version snapshot)
    let approval_uuid = Uuid::new_v4().to_string();
    let approval_id = approval_repo::create_approval(
        pool, &approval_uuid, "course_unpublish", course.id, user_id,
        &req.release_notes, &effective_str, 0, req.notes.as_deref(),
    ).await?;

    // Two-step review: dept_reviewer then admin
    approval_repo::create_step(pool, &Uuid::new_v4().to_string(), approval_id as i64, 1, "dept_reviewer").await?;
    approval_repo::create_step(pool, &Uuid::new_v4().to_string(), approval_id as i64, 2, "admin").await?;

    // Mark course as pending_unpublish
    course_repo::update_course_status(pool, course.id, "pending_unpublish").await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "approval.submit_unpublish",
        "course", Some(course.id), None,
        Some(&serde_json::json!({"release_notes": req.release_notes, "effective_date": req.effective_date})),
        None, None, correlation_id,
    ).await;

    let _ = notification_service::notify_department_role(
        pool, course.department_id, "dept_reviewer",
        "Course Unpublish Request",
        "An unpublish request has been submitted for review.",
        "approval", Some("course"), Some(course_uuid),
    ).await;

    Ok(approval_uuid)
}

pub async fn review_approval(
    pool: &MySqlPool, config: &AppConfig, approval_uuid: &str,
    req: &ReviewApprovalRequest, reviewer_id: i64, reviewer_role: &str,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let approval = approval_repo::find_approval_by_uuid(pool, approval_uuid).await?
        .ok_or_else(|| AppError::NotFound("Approval request not found".to_string()))?;

    // Self-approval prevention: reviewer cannot be the requester
    if approval.requested_by == reviewer_id {
        let _ = security_repo::create_security_event(
            pool, &Uuid::new_v4().to_string(), "self_approval_attempt", "warning",
            Some(reviewer_id), None,
            &format!("User {} attempted to approve their own request {}", reviewer_id, approval_uuid),
            None, correlation_id,
        ).await;
        return Err(AppError::Forbidden("You cannot approve your own submission".to_string()));
    }

    // Get current pending step
    let step = approval_repo::get_current_pending_step(pool, approval.id).await?
        .ok_or_else(|| AppError::Validation("No pending approval step found".to_string()))?;

    // Verify reviewer has appropriate role for this step
    if let Some(required_role) = &step.reviewer_role {
        if reviewer_role != required_role && reviewer_role != "admin" {
            return Err(AppError::Forbidden(format!(
                "This step requires a '{}' reviewer", required_role
            )));
        }
    }

    let step_status = if req.approved { "approved" } else { "rejected" };
    approval_repo::update_step(pool, step.id, reviewer_id, step_status, req.comments.as_deref()).await?;

    if req.approved {
        // Check if there's a next pending step
        let next_step = approval_repo::get_current_pending_step(pool, approval.id).await?;
        if next_step.is_some() {
            // Move to next step
            approval_repo::update_approval_status(pool, approval.id, "pending_step2").await?;
            let _ = notification_service::notify_role(
                pool, "admin",
                "Course Approval: Step 2 Ready",
                "A course approval request has passed Step 1 and requires your final review.",
                "approval", Some("approval"), Some(approval_uuid),
            ).await;
        } else {
            // All steps approved - determine if scheduled or immediate
            let is_unpublish = approval.entity_type == "course_unpublish";
            let target_status = if is_unpublish { "unpublished" } else { "published" };
            let now = chrono::Utc::now().naive_utc();
            let _course = course_repo::find_course_by_id(pool, approval.entity_id).await?
                .ok_or_else(|| AppError::Internal("Course not found".to_string()))?;

            if let Some(effective_date) = approval.effective_date {
                if effective_date > now {
                    // Approved with future effective date — create scheduled transition
                    approval_repo::update_approval_status(pool, approval.id, "approved_scheduled").await?;
                    if !is_unpublish {
                        // Publish: move course to approved_scheduled and record version info
                        course_repo::update_course_status(pool, approval.entity_id, "approved_scheduled").await?;
                        course_repo::update_course_version_info(
                            pool, approval.entity_id, approval.version_number.unwrap_or(1),
                            approval.release_notes.as_deref(),
                            Some(&effective_date.format("%Y-%m-%d %H:%M:%S").to_string()),
                        ).await?;
                    }
                    // For unpublish: course stays pending_unpublish until ScheduledTransition fires
                    approval_repo::create_scheduled_transition(
                        pool, &Uuid::new_v4().to_string(), approval.entity_id, approval.id,
                        target_status, &effective_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                    ).await?;
                } else {
                    // Effective date is in the past — execute immediately
                    approval_repo::update_approval_status(pool, approval.id, "approved").await?;
                    course_repo::update_course_status(pool, approval.entity_id, target_status).await?;
                    if !is_unpublish {
                        course_repo::update_course_version_info(
                            pool, approval.entity_id, approval.version_number.unwrap_or(1),
                            approval.release_notes.as_deref(), None,
                        ).await?;
                    }
                }
            } else {
                // No effective date — execute immediately
                approval_repo::update_approval_status(pool, approval.id, "approved").await?;
                course_repo::update_course_status(pool, approval.entity_id, target_status).await?;
                if !is_unpublish {
                    course_repo::update_course_version_info(
                        pool, approval.entity_id, approval.version_number.unwrap_or(1),
                        approval.release_notes.as_deref(), None,
                    ).await?;
                }
            }
            let _ = notification_service::notify_user(
                pool, approval.requested_by,
                "Course Approved",
                "Your course submission has been approved.",
                "approval", Some("course"), None,
            ).await;
        }
    } else {
        // Rejected: for unpublish restore to published, for publish set rejected
        let restore_status = if approval.entity_type == "course_unpublish" { "published" } else { "rejected" };
        approval_repo::update_approval_status(pool, approval.id, "rejected").await?;
        course_repo::update_course_status(pool, approval.entity_id, restore_status).await?;
        let _ = notification_service::notify_user(
            pool, approval.requested_by,
            "Course Submission Rejected",
            "Your course submission was rejected. Please review the feedback.",
            "approval", Some("course"), None,
        ).await;
    }

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(reviewer_id),
        if req.approved { "approval.approve" } else { "approval.reject" },
        "approval", Some(approval.id), None,
        Some(&serde_json::json!({"step": step.step_order, "comments": req.comments})),
        None, None, correlation_id,
    ).await;

    Ok(())
}

pub async fn get_approval(
    pool: &MySqlPool, uuid: &str,
    role: &str, user_id: i64, department_id: Option<i64>,
) -> Result<ApprovalResponse, AppError> {
    let approval = approval_repo::find_approval_by_uuid(pool, uuid).await?
        .ok_or_else(|| AppError::NotFound("Approval request not found".to_string()))?;

    let active_term_id = term_repo::find_active_term(pool).await
        .map_err(AppError::Database)?
        .map(|t| t.id);

    let can_view = match role {
        "admin" => true,
        _ if approval.requested_by == user_id => true,
        "dept_reviewer" => {
            // Allow if the course belongs to the reviewer's department AND active term
            if let Some(dept_id) = department_id {
                let course = course_repo::find_course_by_id(pool, approval.entity_id).await?;
                course.map_or(false, |c| {
                    c.department_id == Some(dept_id)
                        && term_matches(c.term_id, active_term_id)
                })
            } else {
                false
            }
        }
        _ => false,
    };
    if !can_view {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let steps = approval_repo::get_steps(pool, approval.id).await?;
    Ok(build_approval_response(approval, steps))
}

pub async fn list_approval_queue(
    pool: &MySqlPool, role: &str, department_id: Option<i64>,
) -> Result<Vec<ApprovalQueueItem>, AppError> {
    let active_term_id = term_repo::find_active_term(pool).await
        .map_err(AppError::Database)?
        .map(|t| t.id);

    let approvals = if role == "admin" {
        approval_repo::list_pending_approvals(pool).await?
    } else if let Some(dept_id) = department_id {
        if let Some(term_id) = active_term_id {
            approval_repo::list_pending_for_department_and_term(pool, dept_id, term_id).await?
        } else {
            approval_repo::list_pending_for_department(pool, dept_id).await?
        }
    } else {
        vec![]
    };

    let mut items = Vec::new();
    for approval in approvals {
        let steps = approval_repo::get_steps(pool, approval.id).await?;
        let course = course_repo::find_course_by_id(pool, approval.entity_id).await?;
        let requester = crate::repositories::user_repo::find_by_id(pool, approval.requested_by).await?;

        items.push(ApprovalQueueItem {
            course_title: course.as_ref().map(|c| c.title.clone()).unwrap_or_default(),
            course_code: course.as_ref().map(|c| c.code.clone()).unwrap_or_default(),
            requester_name: requester.as_ref().map(|u| u.full_name.clone()).unwrap_or_default(),
            approval: build_approval_response(approval, steps),
        });
    }
    Ok(items)
}

pub async fn process_scheduled_transitions(pool: &MySqlPool) -> Result<u32, AppError> {
    let transitions = approval_repo::list_pending_transitions(pool).await?;
    let mut count = 0;
    for t in transitions {
        course_repo::update_course_status(pool, t.course_id, &t.target_status).await?;
        approval_repo::mark_transition_executed(pool, t.id).await?;

        let _ = audit_repo::create_audit_log(
            pool, &Uuid::new_v4().to_string(), None, "scheduled_transition.execute",
            "course", Some(t.course_id), None,
            Some(&serde_json::json!({"target_status": t.target_status, "scheduled_at": t.scheduled_at.to_string()})),
            None, None, None,
        ).await;
        count += 1;
    }
    Ok(count)
}

fn parse_effective_date(s: &str) -> Result<NaiveDateTime, AppError> {
    chrono::NaiveDateTime::parse_from_str(s, "%m/%d/%Y %I:%M %p")
        .map_err(|_| AppError::Validation(
            "Invalid effective date format. Use MM/DD/YYYY HH:MM AM/PM".to_string()
        ))
}

fn build_approval_response(a: crate::models::approval::ApprovalRequest, steps: Vec<crate::models::approval::ApprovalStep>) -> ApprovalResponse {
    ApprovalResponse {
        uuid: a.uuid,
        entity_type: a.entity_type,
        entity_id: a.entity_id,
        status: a.status,
        priority: a.priority,
        release_notes: a.release_notes,
        effective_date: a.effective_date.map(|d| d.format("%m/%d/%Y %I:%M %p").to_string()),
        version_number: a.version_number,
        notes: a.notes,
        requested_by: a.requested_by,
        steps: steps.into_iter().map(|s| ApprovalStepResponse {
            uuid: s.uuid,
            step_order: s.step_order,
            reviewer_id: s.reviewer_id,
            reviewer_role: s.reviewer_role,
            status: s.status,
            comments: s.comments,
            decided_at: s.decided_at.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
        }).collect(),
        created_at: a.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }
}
