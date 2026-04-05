use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth::hmac as hmac_util;
use crate::repositories::{webhook_repo, risk_repo, security_repo};
use crate::utils::errors::AppError;

/// Validate that `url` is an approved on-prem endpoint.
///
/// Allowed forms:
/// - Scheme must be `http` or `https`
/// - Host must be one of:
///   - `localhost`
///   - `127.0.0.1`
///   - IPv4 in 10.0.0.0/8
///   - IPv4 in 172.16.0.0/12 (second octet 16–31)
///   - IPv4 in 192.168.0.0/16
///   - Bare hostname: alphanumeric + hyphens only, no dots (e.g. `campuslearn-receiver`)
///
/// Returns `Ok(())` on pass, `Err(reason)` on failure.
pub fn validate_webhook_endpoint(url: &str) -> Result<(), String> {
    // Check scheme
    let rest = if let Some(r) = url.strip_prefix("https://") {
        r
    } else if let Some(r) = url.strip_prefix("http://") {
        r
    } else {
        return Err(format!(
            "webhook endpoint must use http or https scheme; got '{}'",
            url.split("://").next().unwrap_or(url)
        ));
    };

    if rest.is_empty() {
        return Err("webhook endpoint has an empty host".to_string());
    }

    // Extract authority (everything before the first '/')
    let authority = rest.split('/').next().unwrap_or(rest);

    if authority.is_empty() {
        return Err("webhook endpoint has an empty authority".to_string());
    }

    // Strip optional port
    let host = if let Some(bracket_end) = authority.find(']') {
        // IPv6 literal [::1]:port — we don't allow these; fall through to rejection
        &authority[..=bracket_end]
    } else {
        authority.split(':').next().unwrap_or(authority)
    };

    if host.is_empty() {
        return Err("webhook endpoint has an empty host".to_string());
    }

    if is_approved_onprem_host(host) {
        Ok(())
    } else {
        Err(format!(
            "'{}' is not an approved on-prem host; \
             allowed: localhost, 127.0.0.1, private IP ranges (10.x, 172.16-31.x, 192.168.x), \
             or a bare intranet hostname (no dots)",
            host
        ))
    }
}

fn is_approved_onprem_host(host: &str) -> bool {
    if host == "localhost" || host == "127.0.0.1" {
        return true;
    }

    // Attempt to parse as IPv4
    if let Some(ipv4) = parse_ipv4(host) {
        let [a, b, c, d] = ipv4;
        return matches!(
            (a, b, c, d),
            (10, _, _, _)                  // 10.0.0.0/8
            | (172, 16..=31, _, _)         // 172.16.0.0/12
            | (192, 168, _, _)             // 192.168.0.0/16
        );
    }

    // Bare hostname: alphanumeric and hyphens only, no dots
    !host.is_empty()
        && !host.contains('.')
        && host.chars().all(|c| c.is_alphanumeric() || c == '-')
}

fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut octets = [0u8; 4];
    for (i, p) in parts.iter().enumerate() {
        octets[i] = p.parse::<u8>().ok()?;
    }
    Some(octets)
}

pub async fn enqueue_event_webhooks(pool: &MySqlPool, event_type: &str, payload: &serde_json::Value) -> Result<u32, AppError> {
    let subscribers = risk_repo::get_subscribers_for_event(pool, event_type).await?;
    let mut count = 0u32;

    for sub in subscribers {
        if sub.channel != "webhook" {
            continue;
        }
        // Use the per-subscription endpoint; skip if not configured.
        let target_url = match &sub.target_url {
            Some(u) => u.clone(),
            None => continue,
        };
        let payload_str = serde_json::to_string(payload).unwrap_or_default();
        // Sign only when a per-subscription secret is configured.
        let signature = sub.signing_secret.as_deref().map(|secret| {
            hmac_util::compute_signature(secret, &payload_str)
        });

        webhook_repo::enqueue_webhook(
            pool, &Uuid::new_v4().to_string(), sub.id, event_type,
            payload, &target_url, signature.as_deref(),
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

async fn attempt_delivery(url: &str, payload: &serde_json::Value, signature: Option<&str>) -> Result<i32, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let mut builder = client.post(url)
        .header("Content-Type", "application/json")
        .json(payload);

    if let Some(sig) = signature {
        builder = builder.header("X-Webhook-Signature", sig);
    }

    match builder.send().await {
        Ok(resp) => Ok(resp.status().as_u16() as i32),
        Err(e) => Err(e.to_string()),
    }
}
