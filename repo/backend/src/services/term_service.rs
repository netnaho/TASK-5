use sqlx::MySqlPool;
use uuid::Uuid;

use crate::repositories::{term_repo, audit_repo};
use crate::utils::errors::AppError;

pub async fn accept_term(
    pool: &MySqlPool, term_uuid: &str, user_id: i64,
    ip_address: Option<&str>, user_agent: Option<&str>,
) -> Result<(), AppError> {
    let term = term_repo::find_term_by_uuid(pool, term_uuid).await?
        .ok_or_else(|| AppError::NotFound("Term not found".to_string()))?;

    let uuid = Uuid::new_v4().to_string();
    term_repo::accept_term(pool, &uuid, user_id, term.id, ip_address, user_agent).await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "term.accept",
        "term", Some(term.id), None,
        Some(&serde_json::json!({"term_uuid": term_uuid, "term_code": term.code})),
        None, None, None,
    ).await;

    Ok(())
}

pub async fn check_active_term_accepted(pool: &MySqlPool, user_id: i64) -> Result<(), AppError> {
    if let Some(active_term) = term_repo::find_active_term(pool).await? {
        let accepted = term_repo::has_accepted_term(pool, user_id, active_term.id).await?;
        if !accepted {
            return Err(AppError::Forbidden(
                "You must accept the current terms before performing this action. Please accept the terms at POST /api/v1/terms/<term_uuid>/accept".to_string()
            ));
        }
    }
    Ok(())
}
