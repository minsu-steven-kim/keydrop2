use argon2::{Argon2, Algorithm, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{CryptoError, Result};

/// Size of the master key in bytes (256 bits)
pub const MASTER_KEY_SIZE: usize = 32;

/// Size of the salt in bytes (128 bits)
pub const SALT_SIZE: usize = 16;

/// Master key derived from user password
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; MASTER_KEY_SIZE],
}

impl MasterKey {
    pub fn from_bytes(bytes: [u8; MASTER_KEY_SIZE]) -> Self {
        Self { key: bytes }
    }

    pub fn as_bytes(&self) -> &[u8; MASTER_KEY_SIZE] {
        &self.key
    }
}

/// Key set derived from master key for different purposes
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct KeySet {
    /// Key for encrypting vault data
    pub vault_key: [u8; 32],
    /// Key for authentication (e.g., server auth)
    pub auth_key: [u8; 32],
    /// Key for sharing credentials
    pub sharing_key: [u8; 32],
}

/// Salt for key derivation
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Salt {
    bytes: [u8; SALT_SIZE],
}

impl Salt {
    /// Generate a new random salt
    pub fn generate() -> Result<Self> {
        use rand::RngCore;
        let mut bytes = [0u8; SALT_SIZE];
        rand::thread_rng()
            .try_fill_bytes(&mut bytes)
            .map_err(|e| CryptoError::RandomGeneration(e.to_string()))?;
        Ok(Self { bytes })
    }

    /// Create salt from existing bytes
    pub fn from_bytes(bytes: [u8; SALT_SIZE]) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8; SALT_SIZE] {
        &self.bytes
    }
}

/// Derive master key from password using Argon2id
///
/// Uses Argon2id with OWASP-recommended parameters:
/// - Memory: 64 MiB
/// - Iterations: 3
/// - Parallelism: 4
pub fn derive_master_key(password: &str, salt: &Salt) -> Result<MasterKey> {
    // OWASP recommended parameters for Argon2id
    let params = Params::new(
        64 * 1024, // 64 MiB memory
        3,         // 3 iterations
        4,         // 4 parallel lanes
        Some(MASTER_KEY_SIZE),
    )
    .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; MASTER_KEY_SIZE];
    argon2
        .hash_password_into(password.as_bytes(), salt.as_bytes(), &mut key)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    Ok(MasterKey::from_bytes(key))
}

/// Derive multiple keys from master key using HKDF
///
/// Derives three 256-bit keys:
/// - Vault key: for encrypting vault data
/// - Auth key: for server authentication
/// - Sharing key: for secure credential sharing
pub fn derive_keys(master_key: &MasterKey) -> Result<KeySet> {
    let hkdf = Hkdf::<Sha256>::new(None, master_key.as_bytes());

    let mut vault_key = [0u8; 32];
    let mut auth_key = [0u8; 32];
    let mut sharing_key = [0u8; 32];

    hkdf.expand(b"keydrop-vault-key", &mut vault_key)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    hkdf.expand(b"keydrop-auth-key", &mut auth_key)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    hkdf.expand(b"keydrop-sharing-key", &mut sharing_key)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    Ok(KeySet {
        vault_key,
        auth_key,
        sharing_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_master_key() {
        let salt = Salt::generate().unwrap();
        let key1 = derive_master_key("test_password", &salt).unwrap();
        let key2 = derive_master_key("test_password", &salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1.as_bytes(), key2.as_bytes());

        // Different password should produce different key
        let key3 = derive_master_key("different_password", &salt).unwrap();
        assert_ne!(key1.as_bytes(), key3.as_bytes());
    }

    #[test]
    fn test_derive_keys() {
        let salt = Salt::generate().unwrap();
        let master_key = derive_master_key("test_password", &salt).unwrap();
        let key_set = derive_keys(&master_key).unwrap();

        // All keys should be different
        assert_ne!(key_set.vault_key, key_set.auth_key);
        assert_ne!(key_set.vault_key, key_set.sharing_key);
        assert_ne!(key_set.auth_key, key_set.sharing_key);
    }

    #[test]
    fn test_salt_generation() {
        let salt1 = Salt::generate().unwrap();
        let salt2 = Salt::generate().unwrap();

        // Two random salts should be different
        assert_ne!(salt1.as_bytes(), salt2.as_bytes());
    }
}
