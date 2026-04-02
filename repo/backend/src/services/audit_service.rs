use sqlx::MySqlPool;

use crate::models::audit::AuditLog;
use crate::repositories::audit_repo;
use crate::utils::errors::AppError;

pub async fn list_audit_logs(
    pool: &MySqlPool, entity_type: Option<&str>, entity_id: Option<i64>, limit: Option<i64>,
) -> Result<Vec<AuditLog>, AppError> {
    let limit = limit.unwrap_or(100).min(1000);
    let logs = audit_repo::list_audit_logs(pool, entity_type, entity_id, limit).await?;
    Ok(logs)
}
