use sqlx::MySqlPool;
use crate::models::audit::SecurityEvent;

pub async fn create_security_event(
    pool: &MySqlPool, uuid: &str, event_type: &str, severity: &str,
    user_id: Option<i64>, ip_address: Option<&str>, description: &str,
    metadata: Option<&serde_json::Value>, correlation_id: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let meta_str = metadata.map(|v| serde_json::to_string(v).unwrap_or_default());
    let r = sqlx::query(
        "INSERT INTO security_events (uuid, event_type, severity, user_id, ip_address, description, metadata, correlation_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(uuid).bind(event_type).bind(severity).bind(user_id)
    .bind(ip_address).bind(description).bind(meta_str).bind(correlation_id)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn list_security_events(pool: &MySqlPool, limit: i64) -> Result<Vec<SecurityEvent>, sqlx::Error> {
    sqlx::query_as::<_, SecurityEvent>("SELECT * FROM security_events ORDER BY created_at DESC LIMIT ?")
        .bind(limit).fetch_all(pool).await
}
