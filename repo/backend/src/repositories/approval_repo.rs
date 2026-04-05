use sqlx::MySqlPool;
use crate::models::approval::{ApprovalRequest, ApprovalStep, ScheduledTransition};

pub async fn create_approval(
    pool: &MySqlPool, uuid: &str, entity_type: &str, entity_id: i64,
    requested_by: i64, release_notes: &str, effective_date: &str, version_number: i32,
    notes: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO approval_requests (uuid, entity_type, entity_id, requested_by, status, priority, notes, release_notes, effective_date, version_number) VALUES (?, ?, ?, ?, 'pending_step1', 'normal', ?, ?, ?, ?)"
    )
    .bind(uuid).bind(entity_type).bind(entity_id).bind(requested_by)
    .bind(notes).bind(release_notes).bind(effective_date).bind(version_number)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn create_step(pool: &MySqlPool, uuid: &str, request_id: i64, step_order: i32, reviewer_role: &str) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO approval_steps (uuid, approval_request_id, step_order, reviewer_role, status) VALUES (?, ?, ?, ?, 'pending')")
        .bind(uuid).bind(request_id).bind(step_order).bind(reviewer_role)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn find_approval_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<ApprovalRequest>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRequest>("SELECT * FROM approval_requests WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn find_approval_by_id(pool: &MySqlPool, id: i64) -> Result<Option<ApprovalRequest>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRequest>("SELECT * FROM approval_requests WHERE id = ?")
        .bind(id).fetch_optional(pool).await
}

pub async fn list_pending_approvals(pool: &MySqlPool) -> Result<Vec<ApprovalRequest>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRequest>("SELECT * FROM approval_requests WHERE status IN ('pending_step1', 'pending_step2') ORDER BY created_at")
        .fetch_all(pool).await
}

pub async fn list_pending_for_department(pool: &MySqlPool, dept_id: i64) -> Result<Vec<ApprovalRequest>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRequest>(
        "SELECT ar.* FROM approval_requests ar JOIN courses c ON ar.entity_id = c.id AND ar.entity_type IN ('course', 'course_unpublish') WHERE c.department_id = ? AND ar.status IN ('pending_step1', 'pending_step2') ORDER BY ar.created_at"
    ).bind(dept_id).fetch_all(pool).await
}

/// Pending approvals in a department for the active term, plus unscoped courses (term_id IS NULL).
pub async fn list_pending_for_department_and_term(pool: &MySqlPool, dept_id: i64, term_id: i64) -> Result<Vec<ApprovalRequest>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRequest>(
        "SELECT ar.* FROM approval_requests ar \
         JOIN courses c ON ar.entity_id = c.id AND ar.entity_type IN ('course', 'course_unpublish') \
         WHERE c.department_id = ? \
           AND (c.term_id = ? OR c.term_id IS NULL) \
           AND ar.status IN ('pending_step1', 'pending_step2') \
         ORDER BY ar.created_at"
    ).bind(dept_id).bind(term_id).fetch_all(pool).await
}

pub async fn get_steps(pool: &MySqlPool, request_id: i64) -> Result<Vec<ApprovalStep>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalStep>("SELECT * FROM approval_steps WHERE approval_request_id = ? ORDER BY step_order")
        .bind(request_id).fetch_all(pool).await
}

pub async fn get_current_pending_step(pool: &MySqlPool, request_id: i64) -> Result<Option<ApprovalStep>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalStep>("SELECT * FROM approval_steps WHERE approval_request_id = ? AND status = 'pending' ORDER BY step_order LIMIT 1")
        .bind(request_id).fetch_optional(pool).await
}

pub async fn update_step(pool: &MySqlPool, step_id: i64, reviewer_id: i64, status: &str, comments: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE approval_steps SET reviewer_id = ?, status = ?, comments = ?, decided_at = NOW() WHERE id = ?")
        .bind(reviewer_id).bind(status).bind(comments).bind(step_id)
        .execute(pool).await?;
    Ok(())
}

pub async fn update_approval_status(pool: &MySqlPool, id: i64, status: &str) -> Result<(), sqlx::Error> {
    let resolved = if status == "approved" || status == "approved_scheduled" || status == "rejected" { "NOW()" } else { "NULL" };
    // We can't interpolate SQL fragments safely, so use two separate queries
    if status == "approved" || status == "approved_scheduled" || status == "rejected" {
        sqlx::query("UPDATE approval_requests SET status = ?, resolved_at = NOW(), updated_at = NOW() WHERE id = ?")
            .bind(status).bind(id).execute(pool).await?;
    } else {
        sqlx::query("UPDATE approval_requests SET status = ?, updated_at = NOW() WHERE id = ?")
            .bind(status).bind(id).execute(pool).await?;
    }
    Ok(())
}

pub async fn create_scheduled_transition(pool: &MySqlPool, uuid: &str, course_id: i64, approval_id: i64, target_status: &str, scheduled_at: &str) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO scheduled_transitions (uuid, course_id, approval_request_id, target_status, scheduled_at) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid).bind(course_id).bind(approval_id).bind(target_status).bind(scheduled_at)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn list_pending_transitions(pool: &MySqlPool) -> Result<Vec<ScheduledTransition>, sqlx::Error> {
    sqlx::query_as::<_, ScheduledTransition>("SELECT * FROM scheduled_transitions WHERE is_executed = false AND scheduled_at <= NOW()")
        .fetch_all(pool).await
}

pub async fn mark_transition_executed(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE scheduled_transitions SET is_executed = true, executed_at = NOW() WHERE id = ?")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn find_active_approval_for_course(pool: &MySqlPool, course_id: i64) -> Result<Option<ApprovalRequest>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRequest>(
        "SELECT * FROM approval_requests WHERE entity_id = ? AND entity_type IN ('course', 'course_unpublish') AND status IN ('pending_step1', 'pending_step2') LIMIT 1"
    ).bind(course_id).fetch_optional(pool).await
}
