use sqlx::MySqlPool;

/// Check if IP has exceeded rate limits for a given endpoint.
/// Returns true if rate limit is exceeded.
pub async fn check_ip_rate(
    pool: &MySqlPool, ip: &str, endpoint: &str, max_per_minute: i32, max_per_hour: i32,
) -> Result<bool, sqlx::Error> {
    // Check per-minute limit
    let minute_count: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(request_count), 0) FROM ip_rate_limits WHERE ip_address = ? AND endpoint = ? AND window_start > DATE_SUB(NOW(), INTERVAL 1 MINUTE)"
    ).bind(ip).bind(endpoint).fetch_one(pool).await?;

    if minute_count.0 >= max_per_minute as i64 {
        return Ok(true);
    }

    // Check per-hour limit
    let hour_count: (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(request_count), 0) FROM ip_rate_limits WHERE ip_address = ? AND endpoint = ? AND window_start > DATE_SUB(NOW(), INTERVAL 1 HOUR)"
    ).bind(ip).bind(endpoint).fetch_one(pool).await?;

    Ok(hour_count.0 >= max_per_hour as i64)
}

/// Record an IP request for rate limiting.
pub async fn increment_ip_rate(pool: &MySqlPool, ip: &str, endpoint: &str) -> Result<(), sqlx::Error> {
    // Try to increment existing window
    let result = sqlx::query(
        "UPDATE ip_rate_limits SET request_count = request_count + 1 WHERE ip_address = ? AND endpoint = ? AND window_start > DATE_SUB(NOW(), INTERVAL 1 MINUTE)"
    ).bind(ip).bind(endpoint).execute(pool).await?;

    if result.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO ip_rate_limits (ip_address, endpoint, window_start, request_count) VALUES (?, ?, NOW(), 1)"
        ).bind(ip).bind(endpoint).execute(pool).await?;
    }
    Ok(())
}

/// Increment failed login counter for a user. Returns new count.
pub async fn increment_failed_login(pool: &MySqlPool, user_id: i64) -> Result<i32, sqlx::Error> {
    sqlx::query("UPDATE users SET failed_login_count = failed_login_count + 1 WHERE id = ?")
        .bind(user_id).execute(pool).await?;
    let row: (i32,) = sqlx::query_as("SELECT failed_login_count FROM users WHERE id = ?")
        .bind(user_id).fetch_one(pool).await?;
    Ok(row.0)
}

/// Reset failed login counter on successful login.
pub async fn reset_failed_login(pool: &MySqlPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET failed_login_count = 0, locked_until = NULL WHERE id = ?")
        .bind(user_id).execute(pool).await?;
    Ok(())
}

/// Lock account for the given number of minutes.
pub async fn lock_account(pool: &MySqlPool, user_id: i64, duration_minutes: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET locked_until = DATE_ADD(NOW(), INTERVAL ? MINUTE) WHERE id = ?")
        .bind(duration_minutes).bind(user_id).execute(pool).await?;
    Ok(())
}

/// Check if account is currently locked.
pub async fn is_account_locked(pool: &MySqlPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM users WHERE id = ? AND locked_until IS NOT NULL AND locked_until > NOW()"
    ).bind(user_id).fetch_optional(pool).await?;
    Ok(row.is_some())
}

/// Cleanup old IP rate limit entries (called by hourly job).
pub async fn cleanup_old_ip_rates(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM ip_rate_limits WHERE window_start < DATE_SUB(NOW(), INTERVAL 2 HOUR)")
        .execute(pool).await?;
    Ok(result.rows_affected())
}
