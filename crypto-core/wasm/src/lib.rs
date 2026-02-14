//! WASM bindings for Keydrop crypto core
//!
//! This module provides JavaScript-friendly wrappers around the crypto-core library,
//! enabling use in browsers and browser extensions via WebAssembly.

use crypto_core::{
    cipher::{self, EncryptedBlob, KEY_SIZE},
    error::CryptoError,
    kdf::{self, Salt, SALT_SIZE},
    password::{self, PasswordOptions as RustPasswordOptions},
    vault::{Vault as RustVault, VaultItem as RustVaultItem},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Convert CryptoError to JsValue for JavaScript
fn to_js_error(e: CryptoError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

// =============================================================================
// Key Derivation Functions
// =============================================================================

/// Generate a new random salt (16 bytes, returned as base64)
#[wasm_bindgen(js_name = generateSalt)]
pub fn generate_salt() -> Result<String, JsValue> {
    let salt = Salt::generate().map_err(to_js_error)?;
    Ok(base64_encode(salt.as_bytes()))
}

/// Derive master key from password and salt
/// Returns the master key as base64-encoded string
#[wasm_bindgen(js_name = deriveMasterKey)]
pub fn derive_master_key(password: &str, salt_base64: &str) -> Result<String, JsValue> {
    let salt_bytes = base64_decode(salt_base64)?;
    if salt_bytes.len() != SALT_SIZE {
        return Err(JsValue::from_str(&format!(
            "Invalid salt length: expected {}, got {}",
            SALT_SIZE,
            salt_bytes.len()
        )));
    }

    let mut salt_array = [0u8; SALT_SIZE];
    salt_array.copy_from_slice(&salt_bytes);
    let salt = Salt::from_bytes(salt_array);

    let master_key = kdf::derive_master_key(password, &salt).map_err(to_js_error)?;
    Ok(base64_encode(master_key.as_bytes()))
}

/// Derive key set (vault, auth, sharing keys) from master key
/// Returns JSON object with vault_key, auth_key, and sharing_key as base64
#[wasm_bindgen(js_name = deriveKeys)]
pub fn derive_keys(master_key_base64: &str) -> Result<JsValue, JsValue> {
    let master_bytes = base64_decode(master_key_base64)?;
    if master_bytes.len() != KEY_SIZE {
        return Err(JsValue::from_str(&format!(
            "Invalid master key length: expected {}, got {}",
            KEY_SIZE,
            master_bytes.len()
        )));
    }

    let mut master_array = [0u8; KEY_SIZE];
    master_array.copy_from_slice(&master_bytes);
    let master_key = kdf::MasterKey::from_bytes(master_array);

    let keys = kdf::derive_keys(&master_key).map_err(to_js_error)?;

    let result = KeySetJs {
        vault_key: base64_encode(&keys.vault_key),
        auth_key: base64_encode(&keys.auth_key),
        sharing_key: base64_encode(&keys.sharing_key),
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(Serialize)]
struct KeySetJs {
    vault_key: String,
    auth_key: String,
    sharing_key: String,
}

// =============================================================================
// Encryption Functions
// =============================================================================

/// Encrypt data using AES-256-GCM
/// Takes plaintext and key (base64), returns encrypted blob as base64 JSON
#[wasm_bindgen]
pub fn encrypt(plaintext: &str, key_base64: &str) -> Result<String, JsValue> {
    let key = parse_key(key_base64)?;
    let blob = cipher::encrypt(plaintext.as_bytes(), &key).map_err(to_js_error)?;
    Ok(blob.to_base64())
}

/// Decrypt data using AES-256-GCM
/// Takes encrypted blob (base64 JSON) and key (base64), returns plaintext
#[wasm_bindgen]
pub fn decrypt(encrypted_base64: &str, key_base64: &str) -> Result<String, JsValue> {
    let key = parse_key(key_base64)?;
    let blob = EncryptedBlob::from_base64(encrypted_base64).map_err(to_js_error)?;
    let plaintext = cipher::decrypt(&blob, &key).map_err(to_js_error)?;
    String::from_utf8(plaintext).map_err(|e| JsValue::from_str(&e.to_string()))
}

// =============================================================================
// Password Generation
// =============================================================================

/// Password generation options
#[derive(Deserialize)]
pub struct PasswordOptionsJs {
    pub length: Option<usize>,
    pub lowercase: Option<bool>,
    pub uppercase: Option<bool>,
    pub digits: Option<bool>,
    pub symbols: Option<bool>,
    pub exclude_ambiguous: Option<bool>,
    pub exclude_chars: Option<String>,
}

/// Generate a random password with the given options
#[wasm_bindgen(js_name = generatePassword)]
pub fn generate_password(options: JsValue) -> Result<String, JsValue> {
    let opts: PasswordOptionsJs =
        serde_wasm_bindgen::from_value(options).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let rust_opts = RustPasswordOptions {
        length: opts.length.unwrap_or(16),
        lowercase: opts.lowercase.unwrap_or(true),
        uppercase: opts.uppercase.unwrap_or(true),
        digits: opts.digits.unwrap_or(true),
        symbols: opts.symbols.unwrap_or(true),
        exclude_ambiguous: opts.exclude_ambiguous.unwrap_or(false),
        exclude_chars: opts.exclude_chars.unwrap_or_default(),
    };

    password::generate_password(&rust_opts).map_err(to_js_error)
}

/// Generate a passphrase with the given number of words
#[wasm_bindgen(js_name = generatePassphrase)]
pub fn generate_passphrase(word_count: usize, separator: &str) -> Result<String, JsValue> {
    password::generate_passphrase(word_count, separator).map_err(to_js_error)
}

/// Calculate password entropy
#[wasm_bindgen(js_name = calculateEntropy)]
pub fn calculate_entropy(options: JsValue) -> Result<f64, JsValue> {
    let opts: PasswordOptionsJs =
        serde_wasm_bindgen::from_value(options).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let rust_opts = RustPasswordOptions {
        length: opts.length.unwrap_or(16),
        lowercase: opts.lowercase.unwrap_or(true),
        uppercase: opts.uppercase.unwrap_or(true),
        digits: opts.digits.unwrap_or(true),
        symbols: opts.symbols.unwrap_or(true),
        exclude_ambiguous: opts.exclude_ambiguous.unwrap_or(false),
        exclude_chars: opts.exclude_chars.unwrap_or_default(),
    };

    Ok(password::calculate_entropy(&rust_opts))
}

// =============================================================================
// Vault Operations
// =============================================================================

/// Vault item for JavaScript
#[derive(Serialize, Deserialize, Clone)]
pub struct VaultItemJs {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub username: String,
    pub password: String,
    pub notes: Option<String>,
    pub category: Option<String>,
    pub favorite: bool,
    pub created_at: u64,
    pub modified_at: u64,
}

impl From<&RustVaultItem> for VaultItemJs {
    fn from(item: &RustVaultItem) -> Self {
        VaultItemJs {
            id: item.id.clone(),
            name: item.name.clone(),
            url: item.url.clone(),
            username: item.username.clone(),
            password: item.password.clone(),
            notes: item.notes.clone(),
            category: item.category.clone(),
            favorite: item.favorite,
            created_at: item.created_at,
            modified_at: item.modified_at,
        }
    }
}

impl From<VaultItemJs> for RustVaultItem {
    fn from(item: VaultItemJs) -> Self {
        let mut rust_item = RustVaultItem::new(&item.name, &item.username, &item.password);
        rust_item.id = item.id;
        rust_item.url = item.url;
        rust_item.notes = item.notes;
        rust_item.category = item.category;
        rust_item.favorite = item.favorite;
        rust_item.created_at = item.created_at;
        rust_item.modified_at = item.modified_at;
        rust_item
    }
}

/// WASM Vault wrapper
#[wasm_bindgen]
pub struct Vault {
    inner: RustVault,
}

#[wasm_bindgen]
impl Vault {
    /// Create a new empty vault
    #[wasm_bindgen(constructor)]
    pub fn new() -> Vault {
        Vault {
            inner: RustVault::new(),
        }
    }

    /// Add an item to the vault
    #[wasm_bindgen(js_name = addItem)]
    pub fn add_item(&mut self, item: JsValue) -> Result<String, JsValue> {
        let item_js: VaultItemJs =
            serde_wasm_bindgen::from_value(item).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let rust_item: RustVaultItem = item_js.into();
        Ok(self.inner.add_item(rust_item))
    }

    /// Get an item by ID
    #[wasm_bindgen(js_name = getItem)]
    pub fn get_item(&self, id: &str) -> Result<JsValue, JsValue> {
        match self.inner.get_item(id) {
            Some(item) => {
                let item_js = VaultItemJs::from(item);
                serde_wasm_bindgen::to_value(&item_js)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            None => Ok(JsValue::NULL),
        }
    }

    /// Update an item
    #[wasm_bindgen(js_name = updateItem)]
    pub fn update_item(&mut self, id: &str, item: JsValue) -> Result<(), JsValue> {
        let item_js: VaultItemJs =
            serde_wasm_bindgen::from_value(item).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let rust_item: RustVaultItem = item_js.into();
        self.inner.update_item(id, rust_item).map_err(to_js_error)
    }

    /// Remove an item
    #[wasm_bindgen(js_name = removeItem)]
    pub fn remove_item(&mut self, id: &str) -> Result<JsValue, JsValue> {
        let item = self.inner.remove_item(id).map_err(to_js_error)?;
        let item_js = VaultItemJs::from(&item);
        serde_wasm_bindgen::to_value(&item_js).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Search items by query
    #[wasm_bindgen]
    pub fn search(&self, query: &str) -> Result<JsValue, JsValue> {
        let items: Vec<VaultItemJs> = self
            .inner
            .search(query)
            .iter()
            .map(|i| (*i).into())
            .collect();
        serde_wasm_bindgen::to_value(&items).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Find items by URL (for autofill)
    #[wasm_bindgen(js_name = findByUrl)]
    pub fn find_by_url(&self, url: &str) -> Result<JsValue, JsValue> {
        let items: Vec<VaultItemJs> = self
            .inner
            .find_by_url(url)
            .iter()
            .map(|i| (*i).into())
            .collect();
        serde_wasm_bindgen::to_value(&items).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get all items
    #[wasm_bindgen(js_name = getAllItems)]
    pub fn get_all_items(&self) -> Result<JsValue, JsValue> {
        let items: Vec<VaultItemJs> = self.inner.items.iter().map(|i| i.into()).collect();
        serde_wasm_bindgen::to_value(&items).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get categories
    #[wasm_bindgen(js_name = getCategories)]
    pub fn get_categories(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.categories)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get favorites
    #[wasm_bindgen(js_name = getFavorites)]
    pub fn get_favorites(&self) -> Result<JsValue, JsValue> {
        let items: Vec<VaultItemJs> = self
            .inner
            .get_favorites()
            .iter()
            .map(|i| (*i).into())
            .collect();
        serde_wasm_bindgen::to_value(&items).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Export vault as encrypted base64 blob
    #[wasm_bindgen]
    pub fn export(&self, key_base64: &str) -> Result<String, JsValue> {
        let key = parse_key(key_base64)?;
        let blob = self.inner.export(&key).map_err(to_js_error)?;
        Ok(blob.to_base64())
    }

    /// Import vault from encrypted base64 blob
    #[wasm_bindgen(js_name = "import")]
    pub fn import_vault(encrypted_base64: &str, key_base64: &str) -> Result<Vault, JsValue> {
        let key = parse_key(key_base64)?;
        let blob = EncryptedBlob::from_base64(encrypted_base64).map_err(to_js_error)?;
        let inner = RustVault::import(&blob, &key).map_err(to_js_error)?;
        Ok(Vault { inner })
    }

    /// Export vault as JSON (unencrypted, for backup)
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        self.inner.to_json().map_err(to_js_error)
    }

    /// Import vault from JSON
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<Vault, JsValue> {
        let inner = RustVault::from_json(json).map_err(to_js_error)?;
        Ok(Vault { inner })
    }

    /// Get vault item count
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if vault is empty
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(encoded: &str) -> Result<Vec<u8>, JsValue> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| JsValue::from_str(&format!("Base64 decode error: {}", e)))
}

fn parse_key(key_base64: &str) -> Result<[u8; KEY_SIZE], JsValue> {
    let key_bytes = base64_decode(key_base64)?;
    if key_bytes.len() != KEY_SIZE {
        return Err(JsValue::from_str(&format!(
            "Invalid key length: expected {}, got {}",
            KEY_SIZE,
            key_bytes.len()
        )));
    }
    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_generation() {
        let salt = generate_salt().unwrap();
        assert!(!salt.is_empty());
    }
}
