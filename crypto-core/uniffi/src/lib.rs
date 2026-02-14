//! UniFFI bindings for crypto-core
//!
//! Provides Kotlin/Swift bindings for the crypto-core library
//! for use in Android and iOS applications.

use base64::{engine::general_purpose::STANDARD, Engine};
use std::sync::Mutex;

// Re-export crypto_core types
use crypto_core::{
    cipher, kdf,
    password::{self, PasswordOptions as CorePasswordOptions},
    vault::{Vault as CoreVault, VaultItem as CoreVaultItem},
    CryptoError as CoreCryptoError,
};

uniffi::include_scaffolding!("crypto_core");

/// Error type for FFI
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<CoreCryptoError> for CryptoError {
    fn from(e: CoreCryptoError) -> Self {
        match e {
            CoreCryptoError::KeyDerivation(msg) => CryptoError::KeyDerivation(msg),
            CoreCryptoError::Encryption(msg) => CryptoError::Encryption(msg),
            CoreCryptoError::Decryption(msg) => CryptoError::Decryption(msg),
            CoreCryptoError::InvalidKeyLength { .. } => CryptoError::InvalidKeyLength,
            CoreCryptoError::InvalidNonceLength { .. } => {
                CryptoError::InvalidInput("Invalid nonce length".to_string())
            }
            CoreCryptoError::Serialization(msg) => CryptoError::Serialization(msg),
            CoreCryptoError::Deserialization(msg) => CryptoError::Serialization(msg),
            CoreCryptoError::ItemNotFound(msg) => CryptoError::InvalidInput(msg),
            CoreCryptoError::InvalidPasswordOptions(msg) => CryptoError::InvalidInput(msg),
            CoreCryptoError::RandomGeneration(msg) => CryptoError::KeyDerivation(msg),
        }
    }
}

impl From<base64::DecodeError> for CryptoError {
    fn from(e: base64::DecodeError) -> Self {
        CryptoError::InvalidInput(format!("Base64 decode error: {}", e))
    }
}

/// Derived key set
#[derive(Debug, Clone)]
pub struct KeySet {
    pub vault_key: String,
    pub auth_key: String,
    pub sharing_key: String,
}

/// Password generation options
#[derive(Debug, Clone)]
pub struct PasswordOptions {
    pub length: u32,
    pub lowercase: bool,
    pub uppercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
    pub exclude_chars: String,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
            exclude_ambiguous: false,
            exclude_chars: String::new(),
        }
    }
}

impl From<PasswordOptions> for CorePasswordOptions {
    fn from(opts: PasswordOptions) -> Self {
        CorePasswordOptions {
            length: opts.length as usize,
            lowercase: opts.lowercase,
            uppercase: opts.uppercase,
            digits: opts.digits,
            symbols: opts.symbols,
            exclude_ambiguous: opts.exclude_ambiguous,
            exclude_chars: opts.exclude_chars,
        }
    }
}

/// Vault item data for FFI
#[derive(Debug, Clone)]
pub struct VaultItemData {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub username: String,
    pub password: String,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub favorite: bool,
    pub created_at: i64,
    pub modified_at: i64,
}

impl From<&CoreVaultItem> for VaultItemData {
    fn from(item: &CoreVaultItem) -> Self {
        VaultItemData {
            id: item.id.clone(),
            name: item.name.clone(),
            url: item.url.clone(),
            username: item.username.clone(),
            password: item.password.clone(),
            notes: item.notes.clone(),
            category: item.category.clone(),
            favorite: item.favorite,
            created_at: item.created_at as i64,
            modified_at: item.modified_at as i64,
        }
    }
}

impl From<VaultItemData> for CoreVaultItem {
    fn from(data: VaultItemData) -> Self {
        let mut item = CoreVaultItem::new(&data.name, &data.username, &data.password);
        item.id = data.id;
        if let Some(url) = data.url {
            item = item.with_url(&url);
        }
        if let Some(notes) = data.notes {
            item = item.with_notes(&notes);
        }
        if let Some(category) = data.category {
            item = item.with_category(&category);
        }
        item = item.with_favorite(data.favorite);
        item.created_at = data.created_at as u64;
        item.modified_at = data.modified_at as u64;
        item
    }
}

// ============ Free Functions ============

/// Generate a random salt for key derivation
pub fn generate_salt() -> Result<String, CryptoError> {
    let salt = kdf::Salt::generate()?;
    Ok(salt.to_base64())
}

/// Derive master key from password and salt
pub fn derive_master_key(password: String, salt_base64: String) -> Result<String, CryptoError> {
    let salt = kdf::Salt::from_base64(&salt_base64)?;
    let master_key = kdf::derive_master_key(&password, &salt)?;
    Ok(master_key.to_base64())
}

/// Derive encryption keys from master key
pub fn derive_keys(master_key_base64: String) -> Result<KeySet, CryptoError> {
    let master_key_bytes = STANDARD.decode(&master_key_base64)?;
    let master_key = kdf::MasterKey::from_slice(&master_key_bytes)?;
    let keys = kdf::derive_keys(&master_key)?;

    Ok(KeySet {
        vault_key: STANDARD.encode(&keys.vault_key),
        auth_key: STANDARD.encode(&keys.auth_key),
        sharing_key: STANDARD.encode(&keys.sharing_key),
    })
}

/// Encrypt plaintext with key
pub fn encrypt(plaintext: String, key_base64: String) -> Result<String, CryptoError> {
    let key_bytes = STANDARD.decode(&key_base64)?;
    if key_bytes.len() != 32 {
        return Err(CryptoError::InvalidKeyLength);
    }

    let key: [u8; 32] = key_bytes.try_into().unwrap();
    let blob = cipher::encrypt(plaintext.as_bytes(), &key)?;
    Ok(blob.to_base64())
}

/// Decrypt ciphertext with key
pub fn decrypt(encrypted_base64: String, key_base64: String) -> Result<String, CryptoError> {
    let key_bytes = STANDARD.decode(&key_base64)?;
    if key_bytes.len() != 32 {
        return Err(CryptoError::InvalidKeyLength);
    }

    let key: [u8; 32] = key_bytes.try_into().unwrap();
    let blob = cipher::EncryptedBlob::from_base64(&encrypted_base64)?;
    let plaintext = cipher::decrypt(&blob, &key)?;

    String::from_utf8(plaintext)
        .map_err(|e| CryptoError::Decryption(format!("Invalid UTF-8: {}", e)))
}

/// Generate a random password
pub fn generate_password(options: PasswordOptions) -> Result<String, CryptoError> {
    let core_opts: CorePasswordOptions = options.into();
    Ok(password::generate_password(&core_opts)?)
}

/// Generate a passphrase
pub fn generate_passphrase(word_count: u32, separator: String) -> Result<String, CryptoError> {
    Ok(password::generate_passphrase(
        word_count as usize,
        &separator,
    )?)
}

/// Calculate password entropy
pub fn calculate_entropy(options: PasswordOptions) -> f64 {
    let core_opts: CorePasswordOptions = options.into();
    password::calculate_entropy(&core_opts)
}

// ============ Vault Class ============

/// Vault wrapper for FFI
pub struct Vault {
    inner: Mutex<CoreVault>,
}

impl Vault {
    /// Create a new empty vault
    pub fn new() -> Self {
        Vault {
            inner: Mutex::new(CoreVault::new()),
        }
    }

    /// Import vault from encrypted data
    pub fn import_encrypted(
        encrypted_base64: String,
        vault_key_base64: String,
    ) -> Result<Self, CryptoError> {
        let key_bytes = STANDARD.decode(&vault_key_base64)?;
        if key_bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength);
        }

        let key: [u8; 32] = key_bytes.try_into().unwrap();
        let blob = cipher::EncryptedBlob::from_base64(&encrypted_base64)?;
        let vault = CoreVault::import(&blob, &key)?;

        Ok(Vault {
            inner: Mutex::new(vault),
        })
    }

    /// Import vault from JSON
    pub fn from_json(json: String) -> Result<Self, CryptoError> {
        let vault = CoreVault::from_json(&json)?;
        Ok(Vault {
            inner: Mutex::new(vault),
        })
    }

    /// Add an item to the vault
    pub fn add_item(&self, item: VaultItemData) -> Result<String, CryptoError> {
        let mut vault = self.inner.lock().unwrap();
        let core_item: CoreVaultItem = item.into();
        let id = core_item.id.clone();
        vault.add_item(core_item);
        Ok(id)
    }

    /// Get an item by ID
    pub fn get_item(&self, id: String) -> Option<VaultItemData> {
        let vault = self.inner.lock().unwrap();
        vault.get_item(&id).map(VaultItemData::from)
    }

    /// Update an item
    pub fn update_item(&self, id: String, item: VaultItemData) -> Result<(), CryptoError> {
        let mut vault = self.inner.lock().unwrap();
        let core_item: CoreVaultItem = item.into();
        vault.update_item(&id, core_item)?;
        Ok(())
    }

    /// Remove an item
    pub fn remove_item(&self, id: String) -> Result<Option<VaultItemData>, CryptoError> {
        let mut vault = self.inner.lock().unwrap();
        let removed = vault.remove_item(&id)?;
        Ok(Some(VaultItemData::from(&removed)))
    }

    /// Get all items
    pub fn get_all_items(&self) -> Vec<VaultItemData> {
        let vault = self.inner.lock().unwrap();
        vault.items.iter().map(VaultItemData::from).collect()
    }

    /// Search items
    pub fn search(&self, query: String) -> Vec<VaultItemData> {
        let vault = self.inner.lock().unwrap();
        vault
            .search(&query)
            .into_iter()
            .map(VaultItemData::from)
            .collect()
    }

    /// Find items by URL (for autofill)
    pub fn find_by_url(&self, url: String) -> Vec<VaultItemData> {
        let vault = self.inner.lock().unwrap();
        vault
            .find_by_url(&url)
            .into_iter()
            .map(VaultItemData::from)
            .collect()
    }

    /// Get favorite items
    pub fn get_favorites(&self) -> Vec<VaultItemData> {
        let vault = self.inner.lock().unwrap();
        vault
            .get_favorites()
            .into_iter()
            .map(VaultItemData::from)
            .collect()
    }

    /// Get categories
    pub fn get_categories(&self) -> Vec<String> {
        let vault = self.inner.lock().unwrap();
        vault.categories.clone()
    }

    /// Add a category
    pub fn add_category(&self, category: String) -> Result<(), CryptoError> {
        let mut vault = self.inner.lock().unwrap();
        vault.add_category(&category);
        Ok(())
    }

    /// Export encrypted vault
    pub fn export_encrypted(&self, vault_key_base64: String) -> Result<String, CryptoError> {
        let key_bytes = STANDARD.decode(&vault_key_base64)?;
        if key_bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength);
        }

        let key: [u8; 32] = key_bytes.try_into().unwrap();
        let vault = self.inner.lock().unwrap();
        let blob = vault.export(&key)?;
        Ok(blob.to_base64())
    }

    /// Export to JSON (unencrypted)
    pub fn to_json(&self) -> String {
        let vault = self.inner.lock().unwrap();
        vault.to_json().unwrap_or_default()
    }

    /// Get number of items
    pub fn len(&self) -> u32 {
        let vault = self.inner.lock().unwrap();
        vault.items.len() as u32
    }

    /// Check if vault is empty
    pub fn is_empty(&self) -> bool {
        let vault = self.inner.lock().unwrap();
        vault.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let salt = generate_salt().unwrap();
        let master_key = derive_master_key("test_password".to_string(), salt).unwrap();
        let keys = derive_keys(master_key).unwrap();

        assert!(!keys.vault_key.is_empty());
        assert!(!keys.auth_key.is_empty());
        assert!(!keys.sharing_key.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let salt = generate_salt().unwrap();
        let master_key = derive_master_key("test_password".to_string(), salt).unwrap();
        let keys = derive_keys(master_key).unwrap();

        let plaintext = "Hello, World!".to_string();
        let encrypted = encrypt(plaintext.clone(), keys.vault_key.clone()).unwrap();
        let decrypted = decrypt(encrypted, keys.vault_key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_vault_operations() {
        let vault = Vault::new();

        let item = VaultItemData {
            id: String::new(),
            name: "Test".to_string(),
            url: Some("https://example.com".to_string()),
            username: "user".to_string(),
            password: "pass".to_string(),
            notes: None,
            category: None,
            favorite: false,
            created_at: 0,
            modified_at: 0,
        };

        let id = vault.add_item(item).unwrap();
        assert!(!id.is_empty());

        let retrieved = vault.get_item(id.clone()).unwrap();
        assert_eq!(retrieved.name, "Test");

        let all = vault.get_all_items();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_password_generation() {
        let options = PasswordOptions::default();
        let password = generate_password(options).unwrap();
        assert_eq!(password.len(), 20);
    }
}
