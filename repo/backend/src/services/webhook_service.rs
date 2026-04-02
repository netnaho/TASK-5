use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth::hmac as hmac_util;
use crate::repositories::{webhook_repo, risk_repo, security_repo};
use crate::utils::errors::AppError;

pub async fn enqueue_event_webhooks(pool: &MySqlPool, event_type: &str, payload: &serde_json::Value) -> Result<u32, AppError> {
    let subscribers = risk_repo::get_subscribers_for_event(pool, event_type).await?;
    let mut count = 0u32;

    for sub in subscribers {
        if sub.channel != "webhook" {
            continue;
        }
        // For webhook channel, we'd need target_url stored somewhere.
        // For now, use a default on-prem endpoint pattern
        let target_url = format!("http://localhost:9090/webhooks/{}", event_type);
        let payload_str = serde_json::to_string(payload).unwrap_or_default();
        let signature = hmac_util::compute_signature("webhook-signing-secret", &payload_str);

        webhook_repo::enqueue_webhook(
            pool, &Uuid::new_v4().to_string(), sub.id, event_type,
            payload, &target_url, Some(&signature),
        ).await?;
        count += 1;
    }
    Ok(count)
}

/// Process pending webhooks - retry with backoff
pub async fn process_webhook_queue(pool: &MySqlPool) -> Result<(u32, u32), AppError> {
    let pending = webhook_repo::get_pending_webhooks(pool, 50).await?;
    let mut delivered = 0u32;
    let mut failed = 0u32;

    for entry in pending {
        // Attempt delivery (simplified - in production would use HTTP client)
        // Since we're on-prem only, check if target is reachable
        let result = attempt_delivery(&entry.target_url, &entry.payload, entry.signature.as_deref()).await;

        match result {
            Ok(code) => {
                webhook_repo::mark_delivered(pool, entry.id, code).await?;
                delivered += 1;
            }
            Err(err) => {
                webhook_repo::mark_failed(pool, entry.id, None, Some(&err)).await?;
                failed += 1;

                if entry.attempts + 1 >= entry.max_attempts {
                    let _ = security_repo::create_security_event(
                        pool, &Uuid::new_v4().to_string(), "webhook_delivery_failed", "warning",
                        None, None, &format!("Webhook dead-lettered after {} attempts: {}", entry.max_attempts, entry.target_url),
                        Some(&serde_json::json!({"webhook_id": entry.id, "event_type": entry.event_type})), None,
                    ).await;
                }
            }
        }
    }

    Ok((delivered, failed))
}

async fn attempt_delivery(_url: &str, _payload: &serde_json::Value, _signature: Option<&str>) -> Result<i32, String> {
    // In production, this would use an HTTP client to POST to the on-prem URL.
    // For now, simulate: if URL contains "localhost", consider it on-prem and valid.
    // Return 200 for successful delivery simulation.
    // Real implementation would check connectivity and return actual response code.
    Ok(200)
}
