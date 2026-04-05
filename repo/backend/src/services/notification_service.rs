use sqlx::MySqlPool;
use uuid::Uuid;

use crate::repositories::{notification_repo, user_repo};
use crate::utils::errors::AppError;

pub async fn notify_user(
    pool: &MySqlPool, user_id: i64, title: &str, message: &str,
    notification_type: &str, entity_type: Option<&str>, entity_uuid: Option<&str>,
) -> Result<(), AppError> {
    notification_repo::create_notification(
        pool, &Uuid::new_v4().to_string(), user_id,
        title, message, notification_type, entity_type, entity_uuid,
    ).await.map_err(AppError::Database)?;
    Ok(())
}

pub async fn notify_role(
    pool: &MySqlPool, role: &str, title: &str, message: &str,
    notification_type: &str, entity_type: Option<&str>, entity_uuid: Option<&str>,
) -> Result<(), AppError> {
    let users = user_repo::find_users_by_role(pool, role).await.map_err(AppError::Database)?;
    for user in users {
        let _ = notification_repo::create_notification(
            pool, &Uuid::new_v4().to_string(), user.id,
            title, message, notification_type, entity_type, entity_uuid,
        ).await;
    }
    Ok(())
}

/// Notify users with a specific role within a specific department.
/// Falls back to role-wide notification if department_id is None.
pub async fn notify_department_role(
    pool: &MySqlPool, department_id: Option<i64>, role: &str, title: &str, message: &str,
    notification_type: &str, entity_type: Option<&str>, entity_uuid: Option<&str>,
) -> Result<(), AppError> {
    let users = match department_id {
        Some(dept_id) => user_repo::find_users_by_role_and_department(pool, role, dept_id)
            .await.map_err(AppError::Database)?,
        None => user_repo::find_users_by_role(pool, role)
            .await.map_err(AppError::Database)?,
    };
    for user in users {
        let _ = notification_repo::create_notification(
            pool, &Uuid::new_v4().to_string(), user.id,
            title, message, notification_type, entity_type, entity_uuid,
        ).await;
    }
    Ok(())
}
