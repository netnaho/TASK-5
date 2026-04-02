use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::risk::*;
use crate::repositories::{risk_repo, audit_repo, security_repo};
use crate::utils::errors::AppError;

pub async fn list_rules(pool: &MySqlPool) -> Result<Vec<RiskRuleResponse>, AppError> {
    let rules = risk_repo::list_rules(pool).await?;
    Ok(rules.into_iter().map(|r| RiskRuleResponse {
        uuid: r.uuid, name: r.name, description: r.description,
        rule_type: r.rule_type, severity: r.severity, is_active: r.is_active,
        schedule_interval_minutes: r.schedule_interval_minutes,
        last_run_at: r.last_run_at.map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string()),
    }).collect())
}

pub async fn list_events(pool: &MySqlPool, limit: Option<i64>) -> Result<Vec<RiskEventResponse>, AppError> {
    let events = risk_repo::list_risk_events(pool, limit.unwrap_or(100)).await?;
    let mut result = Vec::new();
    for e in events {
        let rule = risk_repo::find_rule_by_id(pool, e.rule_id).await?;
        result.push(RiskEventResponse {
            uuid: e.uuid, rule_id: e.rule_id,
            rule_name: rule.map(|r| r.name),
            entity_type: e.entity_type, entity_id: e.entity_id,
            risk_score: e.risk_score, details: e.details, status: e.status,
            notes: e.notes,
            created_at: e.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        });
    }
    Ok(result)
}

pub async fn update_event(pool: &MySqlPool, event_uuid: &str, req: &UpdateRiskEventRequest, reviewer_id: i64) -> Result<(), AppError> {
    let event = risk_repo::find_risk_event_by_uuid(pool, event_uuid).await?
        .ok_or_else(|| AppError::NotFound("Risk event not found".to_string()))?;

    let valid_statuses = ["acknowledged", "mitigated", "false_positive", "escalated"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return Err(AppError::Validation(format!("Invalid status. Must be one of: {}", valid_statuses.join(", "))));
    }

    risk_repo::update_risk_event_status(pool, event.id, &req.status, Some(reviewer_id), req.notes.as_deref(), req.escalate_to).await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(reviewer_id), "risk_event.review",
        "risk_event", Some(event.id), None,
        Some(&serde_json::json!({"status": req.status, "notes": req.notes})),
        None, None, None,
    ).await;

    Ok(())
}

/// Run all due risk rules - called by scheduled job
pub async fn run_risk_evaluation(pool: &MySqlPool) -> Result<u32, AppError> {
    let rules = risk_repo::get_rules_due_for_run(pool).await?;
    let mut events_created = 0u32;

    for rule in &rules {
        let count = match rule.rule_type.as_str() {
            "posting_frequency" => evaluate_posting_frequency(pool, rule).await?,
            "blacklisted_employer" => evaluate_blacklisted_employers(pool, rule).await?,
            "abnormal_compensation" => evaluate_abnormal_compensation(pool, rule).await?,
            "duplicate_posting" => evaluate_duplicate_postings(pool, rule).await?,
            _ => 0,
        };
        events_created += count;
        risk_repo::update_rule_last_run(pool, rule.id).await?;
    }

    if events_created > 0 {
        tracing::info!(events = events_created, "Risk evaluation created new events");
    }
    Ok(events_created)
}

async fn evaluate_posting_frequency(pool: &MySqlPool, rule: &crate::models::risk::RiskRule) -> Result<u32, AppError> {
    let conditions = &rule.conditions;
    let max_postings = conditions.get("max_postings").and_then(|v| v.as_i64()).unwrap_or(20);
    let window_hours = conditions.get("window_hours").and_then(|v| v.as_i64()).unwrap_or(24);

    // Find employers with high posting frequency
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT employer_name, COUNT(*) as cnt FROM employer_postings WHERE created_at > DATE_SUB(NOW(), INTERVAL ? HOUR) GROUP BY employer_name HAVING cnt > ?"
    ).bind(window_hours).bind(max_postings).fetch_all(pool).await?;

    let mut count = 0;
    for (employer_name, cnt) in rows {
        risk_repo::create_risk_event(
            pool, &Uuid::new_v4().to_string(), rule.id, None,
            Some("employer_posting"), None, (cnt as f64 / max_postings as f64) * 100.0,
            Some(&serde_json::json!({"employer": employer_name, "posting_count": cnt, "window_hours": window_hours})),
        ).await?;
        count += 1;
    }
    Ok(count)
}

async fn evaluate_blacklisted_employers(pool: &MySqlPool, rule: &crate::models::risk::RiskRule) -> Result<u32, AppError> {
    let rows: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT ep.id, ep.employer_name, ep.title, ep.posted_by FROM employer_postings ep INNER JOIN blacklisted_employers be ON ep.employer_name = be.employer_name AND be.is_active = true WHERE ep.flagged = false AND ep.created_at > DATE_SUB(NOW(), INTERVAL 24 HOUR)"
    ).fetch_all(pool).await?;

    let mut count = 0;
    for (posting_id, employer, title, posted_by) in rows {
        risk_repo::create_risk_event(
            pool, &Uuid::new_v4().to_string(), rule.id, Some(posted_by),
            Some("employer_posting"), Some(posting_id), 100.0,
            Some(&serde_json::json!({"employer": employer, "title": title})),
        ).await?;

        // Flag the posting
        sqlx::query("UPDATE employer_postings SET flagged = true WHERE id = ?")
            .bind(posting_id).execute(pool).await?;

        let _ = security_repo::create_security_event(
            pool, &Uuid::new_v4().to_string(), "blacklisted_employer_posting", "critical",
            Some(posted_by), None, &format!("Posting by blacklisted employer: {}", employer),
            None, None,
        ).await;

        count += 1;
    }
    Ok(count)
}

async fn evaluate_abnormal_compensation(pool: &MySqlPool, rule: &crate::models::risk::RiskRule) -> Result<u32, AppError> {
    let conditions = &rule.conditions;
    let min_amount = conditions.get("min_amount").and_then(|v| v.as_f64()).unwrap_or(500.0);
    let max_amount = conditions.get("max_amount").and_then(|v| v.as_f64()).unwrap_or(15000.0);

    let rows: Vec<(i64, String, String, f64, i64)> = sqlx::query_as(
        "SELECT id, employer_name, title, compensation, posted_by FROM employer_postings WHERE posting_type = 'adjunct' AND compensation IS NOT NULL AND (compensation < ? OR compensation > ?) AND flagged = false AND created_at > DATE_SUB(NOW(), INTERVAL 24 HOUR)"
    ).bind(min_amount).bind(max_amount).fetch_all(pool).await?;

    let mut count = 0;
    for (id, employer, title, comp, posted_by) in rows {
        let score = if comp < min_amount { 80.0 } else { 90.0 };
        risk_repo::create_risk_event(
            pool, &Uuid::new_v4().to_string(), rule.id, Some(posted_by),
            Some("employer_posting"), Some(id), score,
            Some(&serde_json::json!({"employer": employer, "title": title, "compensation": comp, "range": format!("{}-{}", min_amount, max_amount)})),
        ).await?;
        sqlx::query("UPDATE employer_postings SET flagged = true WHERE id = ?").bind(id).execute(pool).await?;
        count += 1;
    }
    Ok(count)
}

async fn evaluate_duplicate_postings(pool: &MySqlPool, rule: &crate::models::risk::RiskRule) -> Result<u32, AppError> {
    let conditions = &rule.conditions;
    let window_hours = conditions.get("similarity_window_hours").and_then(|v| v.as_i64()).unwrap_or(48);

    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT employer_name, title, COUNT(*) as cnt FROM employer_postings WHERE created_at > DATE_SUB(NOW(), INTERVAL ? HOUR) AND flagged = false GROUP BY employer_name, title HAVING cnt > 1"
    ).bind(window_hours).fetch_all(pool).await?;

    let mut count = 0;
    for (employer, title, cnt) in rows {
        risk_repo::create_risk_event(
            pool, &Uuid::new_v4().to_string(), rule.id, None,
            Some("employer_posting"), None, 70.0,
            Some(&serde_json::json!({"employer": employer, "title": title, "duplicate_count": cnt})),
        ).await?;
        count += 1;
    }
    Ok(count)
}

// Subscriptions
pub async fn create_subscription(pool: &MySqlPool, user_id: i64, event_type: &str, channel: Option<&str>) -> Result<SubscriptionResponse, AppError> {
    let uuid = Uuid::new_v4().to_string();
    let ch = channel.unwrap_or("in_app");
    risk_repo::create_subscription(pool, &uuid, user_id, event_type, ch).await?;
    Ok(SubscriptionResponse { uuid, event_type: event_type.to_string(), channel: ch.to_string(), is_active: true })
}

pub async fn list_subscriptions(pool: &MySqlPool, user_id: i64) -> Result<Vec<SubscriptionResponse>, AppError> {
    let subs = risk_repo::list_subscriptions(pool, user_id).await?;
    Ok(subs.into_iter().map(|s| SubscriptionResponse {
        uuid: s.uuid, event_type: s.event_type, channel: s.channel, is_active: s.is_active,
    }).collect())
}

// Postings
pub async fn create_posting(pool: &MySqlPool, req: &CreatePostingRequest, user_id: i64) -> Result<String, AppError> {
    // Check blacklist
    if risk_repo::is_employer_blacklisted(pool, &req.employer_name).await? {
        let _ = security_repo::create_security_event(
            pool, &Uuid::new_v4().to_string(), "blacklisted_employer_posting", "critical",
            Some(user_id), None, &format!("Attempted posting by blacklisted employer: {}", req.employer_name),
            None, None,
        ).await;
        return Err(AppError::Forbidden(format!("Employer '{}' is blacklisted", req.employer_name)));
    }

    let uuid = Uuid::new_v4().to_string();
    risk_repo::create_posting(pool, &uuid, &req.employer_name, &req.posting_type, &req.title,
        req.description.as_deref(), req.compensation, user_id).await?;
    Ok(uuid)
}

// Blacklist
pub async fn add_blacklist(pool: &MySqlPool, req: &AddBlacklistRequest, user_id: i64) -> Result<String, AppError> {
    let uuid = Uuid::new_v4().to_string();
    risk_repo::add_blacklisted_employer(pool, &uuid, &req.employer_name, &req.reason, user_id).await?;
    Ok(uuid)
}
