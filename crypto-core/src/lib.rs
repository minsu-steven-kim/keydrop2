//! Keydrop Crypto Core
//!
//! Cryptographic core library for the Keydrop password manager.
//! Provides secure key derivation, encryption, and vault management.
//!
//! # Features
//!
//! - **Key Derivation**: Argon2id for master key derivation, HKDF for key expansion
//! - **Encryption**: AES-256-GCM authenticated encryption
//! - **Vault Management**: Secure storage and retrieval of credentials
//! - **Password Generation**: Configurable random password generation
//!
//! # Example
//!
//! ```rust
//! use crypto_core::{
//!     kdf::{derive_master_key, derive_keys, Salt},
//!     vault::{Vault, VaultItem},
//!     password::{generate_password, PasswordOptions},
//! };
//!
//! // Create a new vault
//! let mut vault = Vault::new();
//!
//! // Generate a strong password
//! let password = generate_password(&PasswordOptions::default()).unwrap();
//!
//! // Add a credential
//! let item = VaultItem::new("GitHub", "user@example.com", &password)
//!     .with_url("https://github.com");
//! vault.add_item(item);
//!
//! // Derive encryption keys from master password
//! let salt = Salt::generate().unwrap();
//! let master_key = derive_master_key("master_password", &salt).unwrap();
//! let keys = derive_keys(&master_key).unwrap();
//!
//! // Export encrypted vault
//! let encrypted = vault.export(&keys.vault_key).unwrap();
//! ```

pub mod cipher;
pub mod error;
pub mod kdf;
pub mod password;
pub mod vault;

// Re-export commonly used types
pub use cipher::{decrypt, encrypt, EncryptedBlob};
pub use error::{CryptoError, Result};
pub use kdf::{derive_keys, derive_master_key, KeySet, MasterKey, Salt};
pub use password::{generate_passphrase, generate_password, PasswordOptions};
pub use vault::{Vault, VaultItem};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        // 1. Generate salt and derive keys
        let salt = Salt::generate().unwrap();
        let master_key = derive_master_key("test_password", &salt).unwrap();
        let keys = derive_keys(&master_key).unwrap();

        // 2. Create vault and add items
        let mut vault = Vault::new();

        let password = generate_password(&PasswordOptions::default()).unwrap();
        let item = VaultItem::new("Test Site", "user@example.com", &password)
            .with_url("https://example.com")
            .with_notes("Test notes")
            .with_category("Login")
            .with_favorite(true);

        vault.add_item(item);

        // 3. Export encrypted vault
        let encrypted = vault.export(&keys.vault_key).unwrap();

        // 4. Import vault with same keys
        let imported = Vault::import(&encrypted, &keys.vault_key).unwrap();

        assert_eq!(imported.items.len(), 1);
        assert_eq!(imported.items[0].name, "Test Site");
        assert_eq!(
            imported.items[0].url,
            Some("https://example.com".to_string())
        );

        // 5. Verify wrong key fails
        let wrong_salt = Salt::generate().unwrap();
        let wrong_master_key = derive_master_key("wrong_password", &wrong_salt).unwrap();
        let wrong_keys = derive_keys(&wrong_master_key).unwrap();

        let result = Vault::import(&encrypted, &wrong_keys.vault_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_and_autofill() {
        let mut vault = Vault::new();

        vault.add_item(
            VaultItem::new("GitHub Personal", "personal@example.com", "pass1")
                .with_url("https://github.com"),
        );
        vault.add_item(
            VaultItem::new("GitHub Work", "work@company.com", "pass2")
                .with_url("https://github.com"),
        );
        vault.add_item(
            VaultItem::new("Google", "user@gmail.com", "pass3")
                .with_url("https://accounts.google.com"),
        );

        // Search
        let results = vault.search("github");
        assert_eq!(results.len(), 2);

        // Find by URL (for autofill)
        let matches = vault.find_by_url("https://github.com/login");
        assert_eq!(matches.len(), 2);

        let matches = vault.find_by_url("https://accounts.google.com/signin");
        assert_eq!(matches.len(), 1);
    }
}
