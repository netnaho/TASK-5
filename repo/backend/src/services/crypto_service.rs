use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;

pub fn encrypt(plaintext: &str, key_hex: &str) -> Result<(String, String), String> {
    let key_bytes = hex::decode(key_hex).map_err(|e| format!("Invalid key hex: {}", e))?;
    if key_bytes.len() != 32 {
        return Err("Key must be 32 bytes (64 hex chars)".to_string());
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok((BASE64.encode(&ciphertext), hex::encode(nonce_bytes)))
}

pub fn decrypt(ciphertext_b64: &str, iv_hex: &str, key_hex: &str) -> Result<String, String> {
    let key_bytes = hex::decode(key_hex).map_err(|e| format!("Invalid key hex: {}", e))?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce_bytes = hex::decode(iv_hex).map_err(|e| format!("Invalid IV hex: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = BASE64.decode(ciphertext_b64).map_err(|e| format!("Invalid base64: {}", e))?;

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}
