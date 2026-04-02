use sqlx::MySqlPool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct WebhookEntry {
    pub id: i64,
    pub uuid: String,
    pub subscription_id: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub target_url: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_attempt_at: Option<chrono::NaiveDateTime>,
    pub next_attempt_at: Option<chrono::NaiveDateTime>,
    pub response_code: Option<i32>,
    pub response_body: Option<String>,
    pub signature: Option<String>,
    pub is_onprem: bool,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn enqueue_webhook(
    pool: &MySqlPool, uuid: &str, subscription_id: i64, event_type: &str,
    payload: &serde_json::Value, target_url: &str, signature: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let payload_str = serde_json::to_string(payload).unwrap_or_default();
    let r = sqlx::query(
        "INSERT INTO webhook_queue (uuid, subscription_id, event_type, payload, target_url, status, signature, is_onprem, next_attempt_at) VALUES (?, ?, ?, ?, ?, 'pending', ?, TRUE, NOW())"
    ).bind(uuid).bind(subscription_id).bind(event_type).bind(payload_str)
    .bind(target_url).bind(signature)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn get_pending_webhooks(pool: &MySqlPool, limit: i64) -> Result<Vec<WebhookEntry>, sqlx::Error> {
    sqlx::query_as::<_, WebhookEntry>(
        "SELECT * FROM webhook_queue WHERE status = 'pending' AND next_attempt_at <= NOW() AND attempts < max_attempts ORDER BY created_at LIMIT ?"
    ).bind(limit).fetch_all(pool).await
}

pub async fn mark_delivered(pool: &MySqlPool, id: i64, response_code: i32) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE webhook_queue SET status = 'delivered', response_code = ?, last_attempt_at = NOW(), attempts = attempts + 1 WHERE id = ?")
        .bind(response_code).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn mark_failed(pool: &MySqlPool, id: i64, response_code: Option<i32>, response_body: Option<&str>) -> Result<(), sqlx::Error> {
    // Exponential backoff: next_attempt = NOW() + (2^attempts * 30 seconds)
    sqlx::query(
        "UPDATE webhook_queue SET status = IF(attempts + 1 >= max_attempts, 'dead_letter', 'pending'), response_code = ?, response_body = ?, last_attempt_at = NOW(), attempts = attempts + 1, next_attempt_at = DATE_ADD(NOW(), INTERVAL POW(2, attempts) * 30 SECOND) WHERE id = ?"
    ).bind(response_code).bind(response_body).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn list_webhooks(pool: &MySqlPool, limit: i64) -> Result<Vec<WebhookEntry>, sqlx::Error> {
    sqlx::query_as::<_, WebhookEntry>("SELECT * FROM webhook_queue ORDER BY created_at DESC LIMIT ?")
        .bind(limit).fetch_all(pool).await
}
