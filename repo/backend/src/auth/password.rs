use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn validate_password_complexity(password: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if password.len() < 12 {
        errors.push("Password must be at least 12 characters long".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain at least one uppercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain at least one lowercase letter".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Password must contain at least one digit".to_string());
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        errors.push("Password must contain at least one special character".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
