use crypto_core::kdf::KeySet;
use crypto_core::vault::Vault;
use std::sync::Mutex;

/// Application state holding the unlocked vault
pub struct AppState {
    /// Currently unlocked vault (None if locked)
    pub vault: Mutex<Option<Vault>>,
    /// Derived keys (None if locked)
    pub keys: Mutex<Option<KeySet>>,
    /// Salt for the current vault (stored separately)
    pub salt: Mutex<Option<[u8; 16]>>,
    /// Auto-lock timeout in seconds
    pub auto_lock_timeout: Mutex<u64>,
    /// Last activity timestamp
    pub last_activity: Mutex<u64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: Mutex::new(None),
            keys: Mutex::new(None),
            salt: Mutex::new(None),
            auto_lock_timeout: Mutex::new(300), // 5 minutes default
            last_activity: Mutex::new(0),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.vault.lock().unwrap().is_some()
    }

    pub fn lock(&self) {
        *self.vault.lock().unwrap() = None;
        *self.keys.lock().unwrap() = None;
    }

    pub fn touch(&self) {
        *self.last_activity.lock().unwrap() = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn should_auto_lock(&self) -> bool {
        let last = *self.last_activity.lock().unwrap();
        let timeout = *self.auto_lock_timeout.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        last > 0 && now - last > timeout
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
