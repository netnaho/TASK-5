use sqlx::MySqlPool;
use crate::models::term::{Term, TermAcceptance};

pub async fn find_active_term(pool: &MySqlPool) -> Result<Option<Term>, sqlx::Error> {
    sqlx::query_as::<_, Term>("SELECT * FROM terms WHERE is_active = TRUE LIMIT 1")
        .fetch_optional(pool).await
}

pub async fn list_terms(pool: &MySqlPool) -> Result<Vec<Term>, sqlx::Error> {
    sqlx::query_as::<_, Term>("SELECT * FROM terms ORDER BY start_date DESC")
        .fetch_all(pool).await
}

pub async fn find_term_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<Term>, sqlx::Error> {
    sqlx::query_as::<_, Term>("SELECT * FROM terms WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn accept_term(pool: &MySqlPool, uuid: &str, user_id: i64, term_id: i64, ip_address: Option<&str>, user_agent: Option<&str>) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO user_term_acceptances (uuid, user_id, term_id, ip_address, user_agent) VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE accepted_at = NOW(), ip_address = VALUES(ip_address), user_agent = VALUES(user_agent)"
    ).bind(uuid).bind(user_id).bind(term_id).bind(ip_address).bind(user_agent)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn has_accepted_term(pool: &MySqlPool, user_id: i64, term_id: i64) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM user_term_acceptances WHERE user_id = ? AND term_id = ? LIMIT 1"
    ).bind(user_id).bind(term_id).fetch_optional(pool).await?;
    Ok(row.is_some())
}

pub async fn get_user_acceptances(pool: &MySqlPool, user_id: i64) -> Result<Vec<TermAcceptance>, sqlx::Error> {
    sqlx::query_as::<_, TermAcceptance>(
        "SELECT * FROM user_term_acceptances WHERE user_id = ? ORDER BY accepted_at DESC"
    ).bind(user_id).fetch_all(pool).await
}
