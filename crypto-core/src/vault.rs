use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cipher::{decrypt, encrypt, EncryptedBlob, KEY_SIZE};
use crate::error::{CryptoError, Result};

/// A single credential item in the vault
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultItem {
    /// Unique identifier for the item
    pub id: String,
    /// Display name for the item
    pub name: String,
    /// Website URL (optional)
    pub url: Option<String>,
    /// Username/email
    pub username: String,
    /// Password (stored encrypted in vault)
    pub password: String,
    /// Additional notes
    pub notes: Option<String>,
    /// Category/folder
    pub category: Option<String>,
    /// Favorite flag
    pub favorite: bool,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// Last modified timestamp (Unix epoch seconds)
    pub modified_at: u64,
    /// Custom fields
    pub custom_fields: Vec<CustomField>,
}

/// Custom field for additional data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomField {
    pub name: String,
    pub value: String,
    pub hidden: bool,
}

impl VaultItem {
    /// Create a new vault item
    pub fn new(name: &str, username: &str, password: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            url: None,
            username: username.to_string(),
            password: password.to_string(),
            notes: None,
            category: None,
            favorite: false,
            created_at: now,
            modified_at: now,
            custom_fields: Vec::new(),
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn with_notes(mut self, notes: &str) -> Self {
        self.notes = Some(notes.to_string());
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    pub fn with_favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }

    pub fn add_custom_field(&mut self, name: &str, value: &str, hidden: bool) {
        self.custom_fields.push(CustomField {
            name: name.to_string(),
            value: value.to_string(),
            hidden,
        });
    }

    fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Vault containing all credential items
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    /// Version for migration purposes
    pub version: u32,
    /// All items in the vault
    pub items: Vec<VaultItem>,
    /// Categories/folders
    pub categories: Vec<String>,
    /// Last sync timestamp (Unix epoch seconds)
    pub last_sync: Option<u64>,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    /// Create a new empty vault
    pub fn new() -> Self {
        Self {
            version: 1,
            items: Vec::new(),
            categories: vec![
                "Login".to_string(),
                "Credit Card".to_string(),
                "Identity".to_string(),
                "Secure Note".to_string(),
            ],
            last_sync: None,
        }
    }

    /// Add an item to the vault
    pub fn add_item(&mut self, item: VaultItem) -> String {
        let id = item.id.clone();
        self.items.push(item);
        id
    }

    /// Get an item by ID
    pub fn get_item(&self, id: &str) -> Option<&VaultItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Get a mutable reference to an item by ID
    pub fn get_item_mut(&mut self, id: &str) -> Option<&mut VaultItem> {
        self.items.iter_mut().find(|item| item.id == id)
    }

    /// Update an item in the vault
    pub fn update_item(&mut self, id: &str, mut updated: VaultItem) -> Result<()> {
        let index = self
            .items
            .iter()
            .position(|item| item.id == id)
            .ok_or_else(|| CryptoError::ItemNotFound(id.to_string()))?;

        updated.id = id.to_string();
        updated.touch();
        self.items[index] = updated;
        Ok(())
    }

    /// Remove an item from the vault
    pub fn remove_item(&mut self, id: &str) -> Result<VaultItem> {
        let index = self
            .items
            .iter()
            .position(|item| item.id == id)
            .ok_or_else(|| CryptoError::ItemNotFound(id.to_string()))?;

        Ok(self.items.remove(index))
    }

    /// Search items by name, URL, or username
    pub fn search(&self, query: &str) -> Vec<&VaultItem> {
        let query_lower = query.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&query_lower)
                    || item.username.to_lowercase().contains(&query_lower)
                    || item
                        .url
                        .as_ref()
                        .map(|u| u.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Find items matching a URL (for autofill)
    pub fn find_by_url(&self, url: &str) -> Vec<&VaultItem> {
        let domain = extract_domain(url);
        self.items
            .iter()
            .filter(|item| {
                item.url
                    .as_ref()
                    .map(|u| {
                        let item_domain = extract_domain(u);
                        domains_match(&domain, &item_domain)
                    })
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get items by category
    pub fn get_by_category(&self, category: &str) -> Vec<&VaultItem> {
        self.items
            .iter()
            .filter(|item| {
                item.category
                    .as_ref()
                    .map(|c| c == category)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get favorite items
    pub fn get_favorites(&self) -> Vec<&VaultItem> {
        self.items.iter().filter(|item| item.favorite).collect()
    }

    /// Add a new category
    pub fn add_category(&mut self, category: &str) {
        if !self.categories.contains(&category.to_string()) {
            self.categories.push(category.to_string());
        }
    }

    /// Export vault to encrypted blob
    pub fn export(&self, key: &[u8; KEY_SIZE]) -> Result<EncryptedBlob> {
        let json =
            serde_json::to_vec(self).map_err(|e| CryptoError::Serialization(e.to_string()))?;
        encrypt(&json, key)
    }

    /// Import vault from encrypted blob
    pub fn import(blob: &EncryptedBlob, key: &[u8; KEY_SIZE]) -> Result<Self> {
        let json = decrypt(blob, key)?;
        serde_json::from_slice(&json).map_err(|e| CryptoError::Deserialization(e.to_string()))
    }

    /// Export vault to JSON string (for backup/transfer)
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| CryptoError::Serialization(e.to_string()))
    }

    /// Import vault from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| CryptoError::Deserialization(e.to_string()))
    }

    /// Get total number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if vault is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    let url = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    url.split('/').next().unwrap_or(url).to_lowercase()
}

/// Check if two domains match (including subdomains)
fn domains_match(domain1: &str, domain2: &str) -> bool {
    if domain1 == domain2 {
        return true;
    }

    // Check if one is a subdomain of the other
    domain1.ends_with(&format!(".{}", domain2)) || domain2.ends_with(&format!(".{}", domain1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    fn test_key() -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    #[test]
    fn test_vault_operations() {
        let mut vault = Vault::new();

        // Add item
        let item = VaultItem::new("Test Site", "user@example.com", "password123")
            .with_url("https://example.com")
            .with_notes("Test notes");

        let id = vault.add_item(item);

        // Get item
        let retrieved = vault.get_item(&id).unwrap();
        assert_eq!(retrieved.name, "Test Site");
        assert_eq!(retrieved.username, "user@example.com");

        // Update item
        let mut updated = retrieved.clone();
        updated.password = "newpassword".to_string();
        vault.update_item(&id, updated).unwrap();

        let retrieved = vault.get_item(&id).unwrap();
        assert_eq!(retrieved.password, "newpassword");

        // Remove item
        let removed = vault.remove_item(&id).unwrap();
        assert_eq!(removed.name, "Test Site");
        assert!(vault.get_item(&id).is_none());
    }

    #[test]
    fn test_vault_search() {
        let mut vault = Vault::new();

        vault.add_item(
            VaultItem::new("GitHub", "dev@example.com", "pass1").with_url("https://github.com"),
        );
        vault.add_item(
            VaultItem::new("GitLab", "dev@example.com", "pass2").with_url("https://gitlab.com"),
        );
        vault.add_item(
            VaultItem::new("Google", "user@gmail.com", "pass3").with_url("https://google.com"),
        );

        // Search by name
        let results = vault.search("git");
        assert_eq!(results.len(), 2);

        // Search by username
        let results = vault.search("gmail");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_vault_find_by_url() {
        let mut vault = Vault::new();

        vault.add_item(
            VaultItem::new("GitHub", "user", "pass").with_url("https://github.com/login"),
        );
        vault.add_item(
            VaultItem::new("GitHub Enterprise", "user", "pass")
                .with_url("https://enterprise.github.com"),
        );

        let results = vault.find_by_url("https://github.com/some/path");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_vault_export_import() {
        let key = test_key();
        let mut vault = Vault::new();

        vault.add_item(VaultItem::new("Test", "user", "password"));

        // Export
        let blob = vault.export(&key).unwrap();

        // Import
        let imported = Vault::import(&blob, &key).unwrap();

        assert_eq!(imported.items.len(), 1);
        assert_eq!(imported.items[0].name, "Test");
        assert_eq!(imported.items[0].password, "password");
    }

    #[test]
    fn test_vault_import_wrong_key() {
        let key1 = test_key();
        let key2 = test_key();
        let mut vault = Vault::new();

        vault.add_item(VaultItem::new("Test", "user", "password"));

        let blob = vault.export(&key1).unwrap();
        let result = Vault::import(&blob, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), "example.com");
        assert_eq!(extract_domain("http://www.example.com"), "example.com");
        assert_eq!(extract_domain("https://sub.example.com"), "sub.example.com");
    }

    #[test]
    fn test_domains_match() {
        assert!(domains_match("example.com", "example.com"));
        assert!(domains_match("sub.example.com", "example.com"));
        assert!(domains_match("example.com", "sub.example.com"));
        assert!(!domains_match("example.com", "other.com"));
    }

    #[test]
    fn test_vault_categories() {
        let mut vault = Vault::new();

        vault.add_item(VaultItem::new("Test", "user", "pass").with_category("Login"));

        let results = vault.get_by_category("Login");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_vault_favorites() {
        let mut vault = Vault::new();

        vault.add_item(VaultItem::new("Test1", "user", "pass").with_favorite(true));
        vault.add_item(VaultItem::new("Test2", "user", "pass").with_favorite(false));

        let results = vault.get_favorites();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Test1");
    }
}
