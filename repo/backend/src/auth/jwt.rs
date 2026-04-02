use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // user uuid
    pub user_id: i64,      // numeric user id for DB queries
    pub username: String,
    pub role: String,
    pub department_id: Option<i64>,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate_token(
    config: &AppConfig,
    user_id: i64,
    user_uuid: &str,
    username: &str,
    role: &str,
    department_id: Option<i64>,
) -> Result<(String, i64), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_in = config.jwt_expiration_hours * 3600;
    let exp = (now + Duration::hours(config.jwt_expiration_hours)).timestamp();

    let claims = Claims {
        sub: user_uuid.to_string(),
        user_id,
        username: username.to_string(),
        role: role.to_string(),
        department_id,
        exp,
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok((token, expires_in))
}

pub fn validate_token(config: &AppConfig, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
