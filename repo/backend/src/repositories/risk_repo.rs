use sqlx::MySqlPool;
use crate::models::risk::*;

pub async fn list_rules(pool: &MySqlPool) -> Result<Vec<RiskRule>, sqlx::Error> {
    sqlx::query_as::<_, RiskRule>("SELECT * FROM risk_rules WHERE is_active = true ORDER BY name")
        .fetch_all(pool).await
}

pub async fn find_rule_by_id(pool: &MySqlPool, id: i64) -> Result<Option<RiskRule>, sqlx::Error> {
    sqlx::query_as::<_, RiskRule>("SELECT * FROM risk_rules WHERE id = ?")
        .bind(id).fetch_optional(pool).await
}

pub async fn get_rules_due_for_run(pool: &MySqlPool) -> Result<Vec<RiskRule>, sqlx::Error> {
    sqlx::query_as::<_, RiskRule>(
        "SELECT * FROM risk_rules WHERE is_active = true AND (last_run_at IS NULL OR last_run_at < DATE_SUB(NOW(), INTERVAL schedule_interval_minutes MINUTE))"
    ).fetch_all(pool).await
}

pub async fn update_rule_last_run(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE risk_rules SET last_run_at = NOW(), updated_at = NOW() WHERE id = ?")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn create_risk_event(
    pool: &MySqlPool, uuid: &str, rule_id: i64, user_id: Option<i64>,
    entity_type: Option<&str>, entity_id: Option<i64>, risk_score: f64,
    details: Option<&serde_json::Value>,
) -> Result<u64, sqlx::Error> {
    let details_str = details.map(|v| serde_json::to_string(v).unwrap_or_default());
    let r = sqlx::query(
        "INSERT INTO risk_events (uuid, rule_id, user_id, entity_type, entity_id, risk_score, details, status) VALUES (?, ?, ?, ?, ?, ?, ?, 'new')"
    ).bind(uuid).bind(rule_id).bind(user_id).bind(entity_type).bind(entity_id)
    .bind(risk_score).bind(details_str)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn list_risk_events(pool: &MySqlPool, limit: i64) -> Result<Vec<RiskEvent>, sqlx::Error> {
    sqlx::query_as::<_, RiskEvent>("SELECT * FROM risk_events ORDER BY created_at DESC LIMIT ?")
        .bind(limit).fetch_all(pool).await
}

pub async fn find_risk_event_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<RiskEvent>, sqlx::Error> {
    sqlx::query_as::<_, RiskEvent>("SELECT * FROM risk_events WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn update_risk_event_status(pool: &MySqlPool, id: i64, status: &str, reviewed_by: Option<i64>, notes: Option<&str>, escalated_to: Option<i64>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE risk_events SET status = ?, reviewed_by = ?, reviewed_at = IF(? IS NOT NULL, NOW(), reviewed_at), notes = COALESCE(?, notes), escalated_to = COALESCE(?, escalated_to) WHERE id = ?")
        .bind(status).bind(reviewed_by).bind(reviewed_by).bind(notes).bind(escalated_to).bind(id)
        .execute(pool).await?;
    Ok(())
}

// Blacklisted employers
pub async fn list_blacklisted_employers(pool: &MySqlPool) -> Result<Vec<BlacklistedEmployer>, sqlx::Error> {
    sqlx::query_as::<_, BlacklistedEmployer>("SELECT * FROM blacklisted_employers WHERE is_active = true ORDER BY employer_name")
        .fetch_all(pool).await
}

pub async fn is_employer_blacklisted(pool: &MySqlPool, name: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM blacklisted_employers WHERE employer_name = ? AND is_active = true LIMIT 1"
    ).bind(name).fetch_optional(pool).await?;
    Ok(row.is_some())
}

pub async fn add_blacklisted_employer(pool: &MySqlPool, uuid: &str, name: &str, reason: &str, added_by: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO blacklisted_employers (uuid, employer_name, reason, added_by) VALUES (?, ?, ?, ?)")
        .bind(uuid).bind(name).bind(reason).bind(added_by)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

// Postings
pub async fn create_posting(pool: &MySqlPool, uuid: &str, employer_name: &str, posting_type: &str, title: &str, description: Option<&str>, compensation: Option<f64>, posted_by: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO employer_postings (uuid, employer_name, posting_type, title, description, compensation, posted_by) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(uuid).bind(employer_name).bind(posting_type).bind(title).bind(description).bind(compensation).bind(posted_by)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn count_postings_in_window(pool: &MySqlPool, employer_name: &str, hours: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM employer_postings WHERE employer_name = ? AND created_at > DATE_SUB(NOW(), INTERVAL ? HOUR)"
    ).bind(employer_name).bind(hours).fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn find_duplicate_postings(pool: &MySqlPool, employer_name: &str, title: &str, hours: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM employer_postings WHERE employer_name = ? AND title = ? AND created_at > DATE_SUB(NOW(), INTERVAL ? HOUR)"
    ).bind(employer_name).bind(title).bind(hours).fetch_one(pool).await?;
    Ok(row.0)
}

// Subscriptions
pub async fn create_subscription(pool: &MySqlPool, uuid: &str, user_id: i64, event_type: &str, channel: &str) -> Result<u64, sqlx::Error> {
    // Use INSERT IGNORE to handle duplicate subscriptions gracefully
    let r = sqlx::query("INSERT IGNORE INTO subscriptions (uuid, user_id, event_type, channel) VALUES (?, ?, ?, ?)")
        .bind(uuid).bind(user_id).bind(event_type).bind(channel)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn list_subscriptions(pool: &MySqlPool, user_id: i64) -> Result<Vec<Subscription>, sqlx::Error> {
    sqlx::query_as::<_, Subscription>("SELECT * FROM subscriptions WHERE user_id = ? AND is_active = true")
        .bind(user_id).fetch_all(pool).await
}

pub async fn get_subscribers_for_event(pool: &MySqlPool, event_type: &str) -> Result<Vec<Subscription>, sqlx::Error> {
    sqlx::query_as::<_, Subscription>("SELECT * FROM subscriptions WHERE event_type = ? AND is_active = true")
        .bind(event_type).fetch_all(pool).await
}

pub async fn delete_subscription(pool: &MySqlPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE subscriptions SET is_active = false WHERE id = ?")
        .bind(id).execute(pool).await?;
    Ok(())
}
