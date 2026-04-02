use sqlx::MySqlPool;
use crate::models::user::User;

pub async fn find_by_username(pool: &MySqlPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? AND is_active = true")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_uuid(pool: &MySqlPool, uuid: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE uuid = ?")
        .bind(uuid)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn count_users(pool: &MySqlPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn create_user(
    pool: &MySqlPool,
    uuid: &str,
    username: &str,
    password_hash: &str,
    email: &str,
    full_name: &str,
    role: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO users (uuid, username, password_hash, email, full_name, role) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid)
    .bind(username)
    .bind(password_hash)
    .bind(email)
    .bind(full_name)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update_password(pool: &MySqlPool, user_id: i64, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = ?, updated_at = NOW() WHERE id = ?")
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_last_login(pool: &MySqlPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_last_reauth(pool: &MySqlPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET last_reauth_at = NOW() WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_by_department(pool: &MySqlPool, department_id: i64) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE department_id = ? AND is_active = true ORDER BY full_name")
        .bind(department_id)
        .fetch_all(pool)
        .await
}
