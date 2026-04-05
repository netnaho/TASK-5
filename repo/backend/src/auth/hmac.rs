use hmac::{Hmac, Mac};
use sha2::Sha256;
use chrono::Utc;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_signature(secret: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn verify_signature(secret: &str, message: &str, signature: &str) -> bool {
    let computed = compute_signature(secret, message);
    // Constant-time comparison
    if computed.len() != signature.len() {
        return false;
    }
    computed.as_bytes().iter()
        .zip(signature.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

pub fn build_signing_message(key_id: &str, nonce: &str, timestamp: i64, method: &str, path: &str) -> String {
    format!("{}:{}:{}:{}:{}", key_id, nonce, timestamp, method, path)
}

pub fn is_timestamp_valid(timestamp: i64, max_age_seconds: i64) -> bool {
    let now = Utc::now().timestamp();
    let diff = (now - timestamp).abs();
    diff <= max_age_seconds
}
