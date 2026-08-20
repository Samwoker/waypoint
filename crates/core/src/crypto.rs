use aes_gcm::{
    aead::Aead,
    Aes256Gcm, KeyInit, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::error::CoreError;

type HmacSha256 = Hmac<Sha256>;

const NONCE_LEN: usize = 12;

/// Encrypts plaintext using AES-256-GCM with the provided 32-byte key.
/// Returns a base64-encoded string containing `nonce + ciphertext + tag`.
pub fn encrypt_secret(plaintext: &[u8], key: &[u8; 32]) -> Result<String, CoreError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CoreError::Crypto(format!("Failed to create cipher: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CoreError::Crypto(format!("Encryption failed: {e}")))?;

    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

/// Decrypts a base64-encoded `nonce + ciphertext + tag` payload using AES-256-GCM with the provided 32-byte key.
pub fn decrypt_secret(encoded: &str, key: &[u8; 32]) -> Result<Vec<u8>, CoreError> {
    let data = BASE64
        .decode(encoded)
        .map_err(|e| CoreError::Crypto(format!("Invalid base64 payload: {e}")))?;

    if data.len() < NONCE_LEN {
        return Err(CoreError::Crypto("Ciphertext is too short".to_string()));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CoreError::Crypto(format!("Failed to create cipher: {e}")))?;

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CoreError::Crypto(format!("Decryption failed: {e}")))
}

/// Signs a message with HMAC-SHA256 and returns the hex-encoded signature.
pub fn sign_hmac_sha256(secret: &[u8], message: &[u8]) -> Result<String, CoreError> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(secret)
        .map_err(|e| CoreError::Crypto(format!("Invalid HMAC key: {e}")))?;
    mac.update(message);
    let result = mac.finalize().into_bytes();
    Ok(hex::encode(result))
}

/// Verifies an HMAC-SHA256 signature using constant-time comparison via the `subtle` crate.
pub fn verify_hmac_sha256(
    secret: &[u8],
    message: &[u8],
    signature_hex: &str,
) -> Result<bool, CoreError> {
    let expected_sig = hex::decode(signature_hex)
        .map_err(|e| CoreError::Crypto(format!("Invalid signature hex: {e}")))?;

    let mut mac = <HmacSha256 as Mac>::new_from_slice(secret)
        .map_err(|e| CoreError::Crypto(format!("Invalid HMAC key: {e}")))?;
    mac.update(message);
    let computed_sig = mac.finalize().into_bytes();

    if expected_sig.len() != computed_sig.len() {
        return Ok(false);
    }

    let is_valid = expected_sig.ct_eq(&computed_sig);
    if is_valid.into() {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Generates a random cryptographic secret of specified byte length, hex-encoded.
pub fn generate_secret(num_bytes: usize) -> String {
    let mut bytes = vec![0u8; num_bytes];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Generates a random cryptographic secret of specified byte length, base64-encoded.
pub fn generate_secret_base64(num_bytes: usize) -> String {
    let mut bytes = vec![0u8; num_bytes];
    rand::thread_rng().fill_bytes(&mut bytes);
    BASE64.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_encryption_and_decryption() {
        let key = [42u8; 32];
        let secret = "whsec_super_secret_webhook_key_123456";

        let encrypted = encrypt_secret(secret.as_bytes(), &key).expect("Encryption failed");
        assert_ne!(encrypted, secret);

        let decrypted_bytes = decrypt_secret(&encrypted, &key).expect("Decryption failed");
        let decrypted = String::from_utf8(decrypted_bytes).expect("UTF-8 decoding failed");

        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_hmac_sign_and_verify() {
        let secret = b"my-signing-secret";
        let message = b"{\"event\":\"payment.succeeded\"}";

        let sig = sign_hmac_sha256(secret, message).expect("Signing failed");
        assert!(verify_hmac_sha256(secret, message, &sig).expect("Verification failed"));
        assert!(!verify_hmac_sha256(secret, b"tampered message", &sig).expect("Verification failed"));
    }
}
