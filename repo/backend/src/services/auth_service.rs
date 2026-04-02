use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth::jwt::generate_token;
use crate::auth::password::{verify_password, hash_password, validate_password_complexity};
use crate::config::AppConfig;
use crate::dto::auth::{LoginResponse, UserInfo};
use crate::repositories::{user_repo, security_repo};
use crate::utils::errors::AppError;

pub async fn login(
    pool: &MySqlPool,
    config: &AppConfig,
    username: &str,
    password: &str,
    ip_address: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<LoginResponse, AppError> {
    let user = user_repo::find_by_username(pool, username)
        .await?
        .ok_or_else(|| {
            // Log failed login for non-existent user
            AppError::Auth("Invalid username or password".to_string())
        });

    let user = match user {
        Ok(u) => u,
        Err(e) => {
            let _ = security_repo::create_security_event(
                pool, &Uuid::new_v4().to_string(), "failed_login", "warning",
                None, ip_address, &format!("Failed login attempt for username: {}", username),
                None, correlation_id,
            ).await;
            return Err(e);
        }
    };

    let valid = verify_password(password, &user.password_hash)
        .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;

    if !valid {
        let _ = security_repo::create_security_event(
            pool, &Uuid::new_v4().to_string(), "failed_login", "warning",
            Some(user.id), ip_address,
            &format!("Failed login: invalid password for user {}", username),
            None, correlation_id,
        ).await;
        return Err(AppError::Auth("Invalid username or password".to_string()));
    }

    let _ = user_repo::update_last_login(pool, user.id).await;

    let (token, expires_in) = generate_token(
        config, user.id, &user.uuid, &user.username, &user.role, user.department_id,
    ).map_err(|e| AppError::Internal(format!("Token generation failed: {e}")))?;

    Ok(LoginResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: UserInfo {
            uuid: user.uuid,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
            department_id: user.department_id,
        },
    })
}

pub async fn change_password(
    pool: &MySqlPool,
    user_id: i64,
    current_password: &str,
    new_password: &str,
    ip_address: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let user = user_repo::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let valid = verify_password(current_password, &user.password_hash)
        .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;

    if !valid {
        let _ = security_repo::create_security_event(
            pool, &Uuid::new_v4().to_string(), "password_change_failed", "warning",
            Some(user_id), ip_address, "Password change failed: invalid current password",
            None, correlation_id,
        ).await;
        return Err(AppError::Auth("Current password is incorrect".to_string()));
    }

    validate_password_complexity(new_password)
        .map_err(|errs| AppError::Validation(errs.join("; ")))?;

    let new_hash = hash_password(new_password)
        .map_err(|_| AppError::Internal("Password hashing failed".to_string()))?;

    user_repo::update_password(pool, user_id, &new_hash).await?;

    let _ = security_repo::create_security_event(
        pool, &Uuid::new_v4().to_string(), "password_changed", "info",
        Some(user_id), ip_address, "Password changed successfully",
        None, correlation_id,
    ).await;

    Ok(())
}

pub async fn reauth(
    pool: &MySqlPool,
    user_id: i64,
    password: &str,
    ip_address: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    let user = user_repo::find_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let valid = verify_password(password, &user.password_hash)
        .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;

    if !valid {
        let _ = security_repo::create_security_event(
            pool, &Uuid::new_v4().to_string(), "reauth_failed", "warning",
            Some(user_id), ip_address, "Re-authentication failed: invalid password",
            None, correlation_id,
        ).await;
        return Err(AppError::Auth("Invalid password".to_string()));
    }

    user_repo::update_last_reauth(pool, user_id).await?;
    Ok(())
}
