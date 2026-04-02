use sqlx::MySqlPool;

pub async fn get_request_count(pool: &MySqlPool, user_id: i64) -> Result<i32, sqlx::Error> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT request_count FROM rate_limit_entries WHERE user_id = ? AND window_start > DATE_SUB(NOW(), INTERVAL 1 MINUTE) ORDER BY window_start DESC LIMIT 1"
    ).bind(user_id).fetch_optional(pool).await?;
    Ok(row.map(|r| r.0).unwrap_or(0))
}

pub async fn increment_request_count(pool: &MySqlPool, user_id: i64) -> Result<i32, sqlx::Error> {
    // Try to update existing window
    let result = sqlx::query(
        "UPDATE rate_limit_entries SET request_count = request_count + 1 WHERE user_id = ? AND window_start > DATE_SUB(NOW(), INTERVAL 1 MINUTE)"
    ).bind(user_id).execute(pool).await?;

    if result.rows_affected() == 0 {
        // Create new window
        sqlx::query("INSERT INTO rate_limit_entries (user_id, window_start, request_count) VALUES (?, NOW(), 1)")
            .bind(user_id).execute(pool).await?;
        return Ok(1);
    }

    let count = get_request_count(pool, user_id).await?;
    Ok(count)
}

pub async fn cleanup_old_entries(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("DELETE FROM rate_limit_entries WHERE window_start < DATE_SUB(NOW(), INTERVAL 5 MINUTE)")
        .execute(pool).await?;
    Ok(r.rows_affected())
}
