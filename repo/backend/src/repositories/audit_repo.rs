use sqlx::MySqlPool;
use crate::models::audit::AuditLog;

pub async fn create_audit_log(
    pool: &MySqlPool, uuid: &str, user_id: Option<i64>, action: &str,
    entity_type: &str, entity_id: Option<i64>,
    old_values: Option<&serde_json::Value>, new_values: Option<&serde_json::Value>,
    ip_address: Option<&str>, user_agent: Option<&str>, correlation_id: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let old_str = old_values.map(|v| serde_json::to_string(v).unwrap_or_default());
    let new_str = new_values.map(|v| serde_json::to_string(v).unwrap_or_default());
    // 7-year retention
    let r = sqlx::query(
        "INSERT INTO audit_logs (uuid, user_id, action, entity_type, entity_id, old_values, new_values, ip_address, user_agent, correlation_id, retention_expires_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, DATE_ADD(NOW(), INTERVAL 7 YEAR))"
    )
    .bind(uuid).bind(user_id).bind(action).bind(entity_type).bind(entity_id)
    .bind(old_str).bind(new_str).bind(ip_address).bind(user_agent).bind(correlation_id)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn list_audit_logs(pool: &MySqlPool, entity_type: Option<&str>, entity_id: Option<i64>, limit: i64) -> Result<Vec<AuditLog>, sqlx::Error> {
    match (entity_type, entity_id) {
        (Some(et), Some(eid)) => {
            sqlx::query_as::<_, AuditLog>("SELECT * FROM audit_logs WHERE entity_type = ? AND entity_id = ? ORDER BY created_at DESC LIMIT ?")
                .bind(et).bind(eid).bind(limit).fetch_all(pool).await
        }
        (Some(et), None) => {
            sqlx::query_as::<_, AuditLog>("SELECT * FROM audit_logs WHERE entity_type = ? ORDER BY created_at DESC LIMIT ?")
                .bind(et).bind(limit).fetch_all(pool).await
        }
        _ => {
            sqlx::query_as::<_, AuditLog>("SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT ?")
                .bind(limit).fetch_all(pool).await
        }
    }
}
