use sqlx::MySqlPool;

use crate::services::{approval_service, risk_service, webhook_service};
use crate::repositories::{rate_limit_repo, login_rate_limit_repo};

/// Process scheduled course transitions (approved_scheduled -> published)
pub async fn run_scheduled_transitions(pool: &MySqlPool) -> Result<u32, Box<dyn std::error::Error>> {
    let count = approval_service::process_scheduled_transitions(pool).await?;
    if count > 0 {
        tracing::info!(count = count, "Processed scheduled transitions");
    }
    Ok(count)
}

/// Evaluate active risk rules that are due for a run.
///
/// Called on every job-loop tick (see `JOB_TICK_SECONDS`). Internally
/// delegates to `risk_service::run_risk_evaluation`, which queries:
///
/// ```sql
/// WHERE last_run_at IS NULL
///    OR last_run_at < NOW() - INTERVAL schedule_interval_minutes MINUTE
/// ```
///
/// Rules not yet due are skipped without side-effects. The default seed sets
/// each rule's `schedule_interval_minutes = 15`, so rules run at most once
/// every 15 minutes regardless of how frequently the loop ticks.
pub async fn run_risk_evaluation(pool: &MySqlPool) -> Result<u32, Box<dyn std::error::Error>> {
    let count = risk_service::run_risk_evaluation(pool).await?;
    if count > 0 {
        tracing::info!(count = count, "Risk evaluation created new events");
    }
    Ok(count)
}

/// Process pending webhook deliveries with retry/backoff
pub async fn process_webhooks(pool: &MySqlPool) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let (delivered, failed) = webhook_service::process_webhook_queue(pool).await?;
    if delivered > 0 || failed > 0 {
        tracing::info!(delivered = delivered, failed = failed, "Webhook processing completed");
    }
    Ok((delivered, failed))
}

/// Clean up expired rate limit entries, nonces, and expired version snapshots
pub async fn cleanup_expired_data(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    let cleaned = rate_limit_repo::cleanup_old_entries(pool).await?;
    if cleaned > 0 {
        tracing::debug!(cleaned = cleaned, "Cleaned up expired rate limit entries");
    }

    // Clean up old IP rate limit entries
    let ip_cleaned = login_rate_limit_repo::cleanup_old_ip_rates(pool).await.unwrap_or(0);
    if ip_cleaned > 0 {
        tracing::debug!(cleaned = ip_cleaned, "Cleaned up old IP rate limit entries");
    }

    sqlx::query("DELETE FROM used_nonces WHERE expires_at < NOW()")
        .execute(pool).await?;

    // Clean expired course versions (180-day retention)
    sqlx::query("DELETE FROM course_versions WHERE expires_at IS NOT NULL AND expires_at < NOW()")
        .execute(pool).await?;

    Ok(())
}
