use sqlx::MySqlPool;
use crate::models::privacy::*;

pub async fn create_data_request(
    pool: &MySqlPool, uuid: &str, user_id: i64, request_type: &str, reason: Option<&str>,
    field_name: Option<&str>, new_value: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO personal_data_requests (uuid, user_id, request_type, status, reason, field_name, new_value) VALUES (?, ?, ?, 'pending', ?, ?, ?)"
    ).bind(uuid).bind(user_id).bind(request_type).bind(reason).bind(field_name).bind(new_value)
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
pub async fn store_encrypted(
    pool: &MySqlPool, uuid: &str, user_id: i64, field_name: &str,
    encrypted_value: &str, iv: &str, key_version: u8,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO sensitive_data_vault \
         (uuid, user_id, field_name, encrypted_value, iv, key_version) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE \
         encrypted_value = VALUES(encrypted_value), \
         iv = VALUES(iv), \
         key_version = VALUES(key_version), \
         updated_at = NOW()"
    )
    .bind(uuid).bind(user_id).bind(field_name)
    .bind(encrypted_value).bind(iv).bind(key_version)
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

/// Gather all user data for export as a JSON value.
pub async fn export_user_data(pool: &MySqlPool, user_id: i64) -> Result<serde_json::Value, sqlx::Error> {
    // User profile
    let user: Option<(String, String, String, String, Option<i64>)> = sqlx::query_as(
        "SELECT username, email, full_name, role, department_id FROM users WHERE id = ?"
    ).bind(user_id).fetch_optional(pool).await?;

    // Bookings
    let bookings: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT uuid, title, start_time, end_time, status FROM bookings WHERE booked_by = ? ORDER BY created_at DESC"
    ).bind(user_id).fetch_all(pool).await?;

    // Courses created
    let courses: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT uuid, title, code, status FROM courses WHERE created_by = ? ORDER BY created_at DESC"
    ).bind(user_id).fetch_all(pool).await?;

    // Audit entries for this user
    let audit_entries: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT uuid, action, created_at FROM audit_logs WHERE user_id = ? ORDER BY created_at DESC LIMIT 500"
    ).bind(user_id).fetch_all(pool).await?;

    // Sensitive fields (names only, not values)
    let vault_fields: Vec<(String,)> = sqlx::query_as(
        "SELECT field_name FROM sensitive_data_vault WHERE user_id = ?"
    ).bind(user_id).fetch_all(pool).await?;

    // Notifications
    let notifications: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT uuid, title, created_at FROM notifications WHERE user_id = ? ORDER BY created_at DESC LIMIT 200"
    ).bind(user_id).fetch_all(pool).await?;

    Ok(serde_json::json!({
        "user": user.map(|(u, e, f, r, d)| serde_json::json!({"username": u, "email": e, "full_name": f, "role": r, "department_id": d})),
        "bookings": bookings.iter().map(|(u, t, s, e, st)| serde_json::json!({"uuid": u, "title": t, "start": s, "end": e, "status": st})).collect::<Vec<_>>(),
        "courses": courses.iter().map(|(u, t, c, s)| serde_json::json!({"uuid": u, "title": t, "code": c, "status": s})).collect::<Vec<_>>(),
        "audit_log_count": audit_entries.len(),
        "audit_entries": audit_entries.iter().map(|(u, a, c)| serde_json::json!({"uuid": u, "action": a, "created_at": c})).collect::<Vec<_>>(),
        "encrypted_fields": vault_fields.iter().map(|(f,)| f.clone()).collect::<Vec<_>>(),
        "notification_count": notifications.len(),
        "exported_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    }))
}

/// Anonymize a user account for deletion compliance.
/// Preserves the row for audit integrity but removes PII.
pub async fn anonymize_user(pool: &MySqlPool, user_id: i64) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Cancel active bookings
    sqlx::query("UPDATE bookings SET status = 'cancelled', updated_at = NOW() WHERE booked_by = ? AND status IN ('confirmed', 'pending')")
        .bind(user_id).execute(&mut *tx).await?;

    // Delete notifications
    sqlx::query("DELETE FROM notifications WHERE user_id = ?")
        .bind(user_id).execute(&mut *tx).await?;

    // Delete sessions
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id).execute(&mut *tx).await?;

    // Delete term acceptances
    sqlx::query("DELETE FROM user_term_acceptances WHERE user_id = ?")
        .bind(user_id).execute(&mut *tx).await?;

    // Delete subscriptions
    sqlx::query("UPDATE subscriptions SET is_active = false WHERE user_id = ?")
        .bind(user_id).execute(&mut *tx).await?;

    // Delete sensitive vault data
    sqlx::query("DELETE FROM sensitive_data_vault WHERE user_id = ?")
        .bind(user_id).execute(&mut *tx).await?;

    // Anonymize user record (preserve for audit trail integrity)
    let anon_username = format!("deleted_{}", user_id);
    let anon_email = format!("deleted_{}@anonymized.local", user_id);
    sqlx::query(
        "UPDATE users SET username = ?, email = ?, full_name = 'Deleted User', password_hash = 'DELETED', is_active = false, updated_at = NOW() WHERE id = ?"
    ).bind(&anon_username).bind(&anon_email).bind(user_id).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

/// Rectify (update) a specific user field. Only allows email and full_name.
/// Returns the old value for audit purposes.
pub async fn rectify_user_field(pool: &MySqlPool, user_id: i64, field_name: &str, new_value: &str) -> Result<String, sqlx::Error> {
    // Fetch old value first
    let old_value: (String,) = match field_name {
        "email" => sqlx::query_as("SELECT email FROM users WHERE id = ?").bind(user_id).fetch_one(pool).await?,
        "full_name" => sqlx::query_as("SELECT full_name FROM users WHERE id = ?").bind(user_id).fetch_one(pool).await?,
        _ => return Err(sqlx::Error::Protocol(format!("Field '{}' cannot be rectified", field_name))),
    };

    match field_name {
        "email" => {
            sqlx::query("UPDATE users SET email = ?, updated_at = NOW() WHERE id = ?")
                .bind(new_value).bind(user_id).execute(pool).await?;
        }
        "full_name" => {
            sqlx::query("UPDATE users SET full_name = ?, updated_at = NOW() WHERE id = ?")
                .bind(new_value).bind(user_id).execute(pool).await?;
        }
        _ => {}
    }

    Ok(old_value.0)
}
