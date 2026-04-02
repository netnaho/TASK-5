use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(length(min = 12, message = "Password must be at least 12 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub department_id: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 12, message = "Current password must be at least 12 characters"))]
    pub current_password: String,
    #[validate(length(min = 12, message = "New password must be at least 12 characters"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ReauthRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct HmacSignedRequest {
    pub key_id: String,
    pub nonce: String,
    pub timestamp: i64,
    pub signature: String,
    pub body: Option<serde_json::Value>,
}
