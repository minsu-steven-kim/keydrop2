use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::{CryptoError, Result};

/// Size of the AES-GCM nonce in bytes (96 bits)
pub const NONCE_SIZE: usize = 12;

/// Size of the AES-256 key in bytes (256 bits)
pub const KEY_SIZE: usize = 32;

/// Encrypted data blob containing ciphertext and nonce
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EncryptedBlob {
    /// Random nonce used for this encryption
    pub nonce: [u8; NONCE_SIZE],
    /// Ciphertext with authentication tag
    pub ciphertext: Vec<u8>,
}

impl EncryptedBlob {
    /// Encode to base64 string for storage
    pub fn to_base64(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, json)
    }

    /// Decode from base64 string
    pub fn from_base64(encoded: &str) -> Result<Self> {
        let json = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|e| CryptoError::Deserialization(e.to_string()))?;
        serde_json::from_slice(&json).map_err(|e| CryptoError::Deserialization(e.to_string()))
    }
}

/// Encrypt data using AES-256-GCM
///
/// Generates a random 96-bit nonce for each encryption.
/// Returns an EncryptedBlob containing the nonce and ciphertext.
pub fn encrypt(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<EncryptedBlob> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::Encryption(e.to_string()))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng()
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| CryptoError::RandomGeneration(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    Ok(EncryptedBlob {
        nonce: nonce_bytes,
        ciphertext,
    })
}

/// Decrypt an EncryptedBlob using AES-256-GCM
///
/// Verifies the authentication tag and returns the plaintext.
pub fn decrypt(blob: &EncryptedBlob, key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::Decryption(e.to_string()))?;

    let nonce = Nonce::from_slice(&blob.nonce);

    cipher
        .decrypt(nonce, blob.ciphertext.as_ref())
        .map_err(|e| CryptoError::Decryption(e.to_string()))
}

/// Encrypt a string and return base64-encoded blob
pub fn encrypt_string(plaintext: &str, key: &[u8; KEY_SIZE]) -> Result<String> {
    let blob = encrypt(plaintext.as_bytes(), key)?;
    Ok(blob.to_base64())
}

/// Decrypt a base64-encoded blob and return string
pub fn decrypt_string(encoded: &str, key: &[u8; KEY_SIZE]) -> Result<String> {
    let blob = EncryptedBlob::from_base64(encoded)?;
    let plaintext = decrypt(&blob, key)?;
    String::from_utf8(plaintext).map_err(|e| CryptoError::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let blob = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&blob, &key).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let blob1 = encrypt(plaintext, &key).unwrap();
        let blob2 = encrypt(plaintext, &key).unwrap();

        // Same plaintext should produce different ciphertext (different nonces)
        assert_ne!(blob1.ciphertext, blob2.ciphertext);
        assert_ne!(blob1.nonce, blob2.nonce);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = test_key();
        let key2 = test_key();
        let plaintext = b"Hello, World!";

        let blob = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&blob, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = test_key();
        let plaintext = "Secret message with unicode: 你好世界 🔐";

        let encoded = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encoded, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_base64_roundtrip() {
        let key = test_key();
        let plaintext = b"Test data";

        let blob = encrypt(plaintext, &key).unwrap();
        let encoded = blob.to_base64();
        let decoded = EncryptedBlob::from_base64(&encoded).unwrap();

        assert_eq!(blob.nonce, decoded.nonce);
        assert_eq!(blob.ciphertext, decoded.ciphertext);
    }
}
