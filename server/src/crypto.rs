use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;

use crate::error::AppError;

pub const KEY_LEN: usize = 32;

const NONCE_LEN: usize = 12;

pub fn encrypt(key: &[u8; KEY_LEN], plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
    let cipher = Aes256Gcm::new(key.into());

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| AppError::Internal("Encryption failed".to_string()))?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    Ok(out)
}

pub fn decrypt(key: &[u8; KEY_LEN], blob: &[u8]) -> Result<Vec<u8>, AppError> {
    if blob.len() < NONCE_LEN {
        return Err(AppError::Internal("Ciphertext too short".to_string()));
    }

    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(key.into());

    cipher.decrypt(nonce, ciphertext).map_err(|_| {
        AppError::Internal("Decryption failed (tampered ciphertext or wrong key)".to_string())
    })
}

pub fn parse_key_from_hex(hex_str: &str) -> Result<[u8; KEY_LEN], String> {
    let bytes = hex::decode(hex_str).map_err(|_| "Master key must be valid hex".to_string())?;

    if bytes.len() != KEY_LEN {
        return Err(format!(
            "Master key must be {} bytes ({} hex chars), got {} bytes",
            KEY_LEN,
            KEY_LEN * 2,
            bytes.len()
        ));
    }

    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&bytes);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; KEY_LEN] {
        [0x42; KEY_LEN]
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"ghp_MySecretGitHubToken123456789";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_ciphertexts_for_same_plaintext() {
        let key = test_key();
        let plaintext = b"hello";

        let a = encrypt(&key, plaintext).unwrap();
        let b = encrypt(&key, plaintext).unwrap();

        assert_ne!(a, b);
    }

    #[test]
    fn decrypt_fails_on_tampered_ciphertext() {
        let key = test_key();
        let plaintext = b"secret token";

        let mut ciphertext = encrypt(&key, plaintext).unwrap();
        ciphertext[NONCE_LEN + 2] ^= 0x01;

        assert!(decrypt(&key, &ciphertext).is_err());
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let key = test_key();
        let wrong_key = [0x99; KEY_LEN];
        let plaintext = b"secret token";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        assert!(decrypt(&wrong_key, &ciphertext).is_err());
    }

    #[test]
    fn parse_key_from_hex_valid() {
        let hex = "0000000000000000000000000000000000000000000000000000000000000042";
        let key = parse_key_from_hex(hex).unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(key[31], 0x42);
    }

    #[test]
    fn parse_key_from_hex_wrong_length() {
        assert!(parse_key_from_hex("abcdef").is_err());
    }

    #[test]
    fn parse_key_from_hex_invalid_chars() {
        let hex = "gg00000000000000000000000000000000000000000000000000000000000042";
        assert!(parse_key_from_hex(hex).is_err());
    }
}
