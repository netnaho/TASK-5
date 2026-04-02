use sqlx::MySqlPool;
use crate::models::privacy::*;

pub async fn create_data_request(pool: &MySqlPool, uuid: &str, user_id: i64, request_type: &str, reason: Option<&str>) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("INSERT INTO personal_data_requests (uuid, user_id, request_type, status, reason) VALUES (?, ?, ?, 'pending', ?)")
        .bind(uuid).bind(user_id).bind(request_type).bind(reason)
        .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn find_data_request_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<PersonalDataRequest>, sqlx::Error> {
    sqlx::query_as::<_, PersonalDataRequest>("SELECT * FROM personal_data_requests WHERE uuid = ?")
        .bind(uuid).fetch_optional(pool).await
}

pub async fn list_data_requests(pool: &MySqlPool, status: Option<&str>) -> Result<Vec<PersonalDataRequest>, sqlx::Error> {
    match status {
        Some(s) => sqlx::query_as::<_, PersonalDataRequest>("SELECT * FROM personal_data_requests WHERE status = ? ORDER BY created_at DESC")
            .bind(s).fetch_all(pool).await,
        None => sqlx::query_as::<_, PersonalDataRequest>("SELECT * FROM personal_data_requests ORDER BY created_at DESC")
            .fetch_all(pool).await,
    }
}

pub async fn list_user_data_requests(pool: &MySqlPool, user_id: i64) -> Result<Vec<PersonalDataRequest>, sqlx::Error> {
    sqlx::query_as::<_, PersonalDataRequest>("SELECT * FROM personal_data_requests WHERE user_id = ? ORDER BY created_at DESC")
        .bind(user_id).fetch_all(pool).await
}

pub async fn approve_data_request(pool: &MySqlPool, id: i64, approved_by: i64, notes: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE personal_data_requests SET status = 'processing', approved_by = ?, approved_at = NOW(), admin_notes = ?, updated_at = NOW() WHERE id = ?")
        .bind(approved_by).bind(notes).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn reject_data_request(pool: &MySqlPool, id: i64, processed_by: i64, notes: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE personal_data_requests SET status = 'rejected', processed_by = ?, processed_at = NOW(), admin_notes = ?, updated_at = NOW() WHERE id = ?")
        .bind(processed_by).bind(notes).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn complete_data_request(pool: &MySqlPool, id: i64, processed_by: i64, result_path: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE personal_data_requests SET status = 'completed', processed_by = ?, processed_at = NOW(), result_file_path = ?, updated_at = NOW() WHERE id = ?")
        .bind(processed_by).bind(result_path).bind(id).execute(pool).await?;
    Ok(())
}

// Sensitive data vault
pub async fn store_encrypted(pool: &MySqlPool, uuid: &str, user_id: i64, field_name: &str, encrypted_value: &str, iv: &str) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO sensitive_data_vault (uuid, user_id, field_name, encrypted_value, iv) VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE encrypted_value = VALUES(encrypted_value), iv = VALUES(iv), updated_at = NOW()"
    ).bind(uuid).bind(user_id).bind(field_name).bind(encrypted_value).bind(iv)
    .execute(pool).await?;
    Ok(r.last_insert_id())
}

pub async fn get_encrypted(pool: &MySqlPool, user_id: i64, field_name: &str) -> Result<Option<SensitiveDataVault>, sqlx::Error> {
    sqlx::query_as::<_, SensitiveDataVault>("SELECT * FROM sensitive_data_vault WHERE user_id = ? AND field_name = ?")
        .bind(user_id).bind(field_name).fetch_optional(pool).await
}

pub async fn list_encrypted_fields(pool: &MySqlPool, user_id: i64) -> Result<Vec<SensitiveDataVault>, sqlx::Error> {
    sqlx::query_as::<_, SensitiveDataVault>("SELECT * FROM sensitive_data_vault WHERE user_id = ?")
        .bind(user_id).fetch_all(pool).await
}

pub async fn delete_user_sensitive_data(pool: &MySqlPool, user_id: i64) -> Result<u64, sqlx::Error> {
    let r = sqlx::query("DELETE FROM sensitive_data_vault WHERE user_id = ?")
        .bind(user_id).execute(pool).await?;
    Ok(r.rows_affected())
}
