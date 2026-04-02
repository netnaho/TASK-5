use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::privacy::*;
use crate::repositories::{privacy_repo, audit_repo};
use crate::services::crypto_service;
use crate::utils::errors::AppError;

pub async fn create_data_request(pool: &MySqlPool, user_id: i64, req: &CreateDataRequest) -> Result<String, AppError> {
    let valid_types = ["export", "delete", "rectify"];
    if !valid_types.contains(&req.request_type.as_str()) {
        return Err(AppError::Validation("Invalid request type. Must be: export, delete, or rectify".to_string()));
    }
    let uuid = Uuid::new_v4().to_string();
    privacy_repo::create_data_request(pool, &uuid, user_id, &req.request_type, req.reason.as_deref()).await?;

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(user_id), "privacy.request_created",
        "personal_data_request", None, None,
        Some(&serde_json::json!({"type": req.request_type})),
        None, None, None,
    ).await;

    Ok(uuid)
}

pub async fn admin_review_request(pool: &MySqlPool, request_uuid: &str, req: &AdminReviewDataRequest, admin_id: i64) -> Result<(), AppError> {
    let data_req = privacy_repo::find_data_request_by_uuid(pool, request_uuid).await?
        .ok_or_else(|| AppError::NotFound("Data request not found".to_string()))?;

    if data_req.status != "pending" {
        return Err(AppError::Validation(format!("Request is already in '{}' status", data_req.status)));
    }

    if req.approved {
        privacy_repo::approve_data_request(pool, data_req.id, admin_id, req.admin_notes.as_deref()).await?;

        // Process the request
        match data_req.request_type.as_str() {
            "export" => {
                let path = format!("/data/exports/user_{}_export_{}.json", data_req.user_id, Uuid::new_v4());
                privacy_repo::complete_data_request(pool, data_req.id, admin_id, Some(&path)).await?;
            }
            "delete" => {
                privacy_repo::delete_user_sensitive_data(pool, data_req.user_id).await?;
                privacy_repo::complete_data_request(pool, data_req.id, admin_id, None).await?;
            }
            "rectify" => {
                privacy_repo::complete_data_request(pool, data_req.id, admin_id, None).await?;
            }
            _ => {}
        }
    } else {
        privacy_repo::reject_data_request(pool, data_req.id, admin_id, req.admin_notes.as_deref()).await?;
    }

    let _ = audit_repo::create_audit_log(
        pool, &Uuid::new_v4().to_string(), Some(admin_id),
        if req.approved { "privacy.request_approved" } else { "privacy.request_rejected" },
        "personal_data_request", Some(data_req.id), None,
        Some(&serde_json::json!({"type": data_req.request_type, "approved": req.approved})),
        None, None, None,
    ).await;

    Ok(())
}

pub async fn list_requests(pool: &MySqlPool, status: Option<&str>) -> Result<Vec<DataRequestResponse>, AppError> {
    let requests = privacy_repo::list_data_requests(pool, status).await?;
    Ok(requests.into_iter().map(|r| DataRequestResponse {
        uuid: r.uuid, user_id: r.user_id, request_type: r.request_type,
        status: r.status, reason: r.reason, admin_notes: r.admin_notes,
        result_file_path: r.result_file_path,
        created_at: r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        updated_at: r.updated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }).collect())
}

pub async fn list_user_requests(pool: &MySqlPool, user_id: i64) -> Result<Vec<DataRequestResponse>, AppError> {
    let requests = privacy_repo::list_user_data_requests(pool, user_id).await?;
    Ok(requests.into_iter().map(|r| DataRequestResponse {
        uuid: r.uuid, user_id: r.user_id, request_type: r.request_type,
        status: r.status, reason: r.reason, admin_notes: r.admin_notes,
        result_file_path: r.result_file_path,
        created_at: r.created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        updated_at: r.updated_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
    }).collect())
}

pub async fn store_sensitive_field(pool: &MySqlPool, user_id: i64, field_name: &str, value: &str, encryption_key: &str) -> Result<(), AppError> {
    let (encrypted, iv) = crypto_service::encrypt(value, encryption_key)
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;
    privacy_repo::store_encrypted(pool, &Uuid::new_v4().to_string(), user_id, field_name, &encrypted, &iv).await?;
    Ok(())
}

pub async fn get_masked_fields(pool: &MySqlPool, user_id: i64) -> Result<Vec<MaskedFieldResponse>, AppError> {
    let fields = privacy_repo::list_encrypted_fields(pool, user_id).await?;
    Ok(fields.into_iter().map(|f| MaskedFieldResponse {
        field_name: f.field_name.clone(),
        masked_value: mask_value(&f.field_name, &f.encrypted_value),
    }).collect())
}

fn mask_value(field_name: &str, _encrypted: &str) -> String {
    match field_name {
        "ssn" => "***-**-####".to_string(),
        "bank_account" => "****####".to_string(),
        "bank_routing" => "****####".to_string(),
        _ => "********".to_string(),
    }
}
