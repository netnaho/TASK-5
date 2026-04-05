use sqlx::MySqlPool;
use crate::models::notification::Notification;

pub async fn create_notification(
    pool: &MySqlPool, uuid: &str, user_id: i64, title: &str, message: &str,
    notification_type: &str, entity_type: Option<&str>, entity_uuid: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO notifications (uuid, user_id, title, message, notification_type, entity_type, entity_uuid) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(uuid).bind(user_id).bind(title).bind(message)
    .bind(notification_type).bind(entity_type).bind(entity_uuid)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn list_for_user(pool: &MySqlPool, user_id: i64, limit: i64) -> Result<Vec<Notification>, sqlx::Error> {
    sqlx::query_as::<_, Notification>(
        "SELECT * FROM notifications WHERE user_id = ? ORDER BY created_at DESC LIMIT ?"
    ).bind(user_id).bind(limit).fetch_all(pool).await
}

pub async fn get_unread_count(pool: &MySqlPool, user_id: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications WHERE user_id = ? AND is_read = false"
    ).bind(user_id).fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn mark_read(pool: &MySqlPool, uuid: &str, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE uuid = ? AND user_id = ?")
        .bind(uuid).bind(user_id).execute(pool).await?;
    Ok(())
}

pub async fn mark_all_read(pool: &MySqlPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET is_read = true WHERE user_id = ? AND is_read = false")
        .bind(user_id).execute(pool).await?;
    Ok(())
}
