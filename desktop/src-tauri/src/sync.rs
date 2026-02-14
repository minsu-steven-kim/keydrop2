use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Sync status state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatusState {
    Idle,
    Syncing,
    Error,
    Offline,
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub state: SyncStatusState,
    pub last_sync_time: Option<u64>,
    pub error: Option<String>,
    pub pending_changes: u32,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            state: SyncStatusState::Idle,
            last_sync_time: None,
            error: None,
            pending_changes: 0,
        }
    }
}

/// Remote command from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommand {
    pub id: String,
    pub command_type: String, // "lock" or "wipe"
    pub created_at: u64,
}

/// Sync state manager
pub struct SyncState {
    pub status: Mutex<SyncStatus>,
    pub is_enabled: Mutex<bool>,
    pub server_url: Mutex<Option<String>>,
    pub access_token: Mutex<Option<String>>,
    pub device_id: Mutex<Option<String>>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(SyncStatus::default()),
            is_enabled: Mutex::new(false),
            server_url: Mutex::new(None),
            access_token: Mutex::new(None),
            device_id: Mutex::new(None),
        }
    }

    pub fn get_status(&self) -> SyncStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_syncing(&self) {
        let mut status = self.status.lock().unwrap();
        status.state = SyncStatusState::Syncing;
        status.error = None;
    }

    pub fn set_idle(&self, last_sync_time: u64) {
        let mut status = self.status.lock().unwrap();
        status.state = SyncStatusState::Idle;
        status.last_sync_time = Some(last_sync_time);
        status.error = None;
    }

    pub fn set_error(&self, error: String) {
        let mut status = self.status.lock().unwrap();
        status.state = SyncStatusState::Error;
        status.error = Some(error);
    }

    pub fn set_offline(&self) {
        let mut status = self.status.lock().unwrap();
        status.state = SyncStatusState::Offline;
    }

    pub fn set_pending_changes(&self, count: u32) {
        let mut status = self.status.lock().unwrap();
        status.pending_changes = count;
    }

    pub fn is_enabled(&self) -> bool {
        *self.is_enabled.lock().unwrap()
    }

    pub fn enable(&self, server_url: String, access_token: String, device_id: String) {
        *self.is_enabled.lock().unwrap() = true;
        *self.server_url.lock().unwrap() = Some(server_url);
        *self.access_token.lock().unwrap() = Some(access_token);
        *self.device_id.lock().unwrap() = Some(device_id);
    }

    pub fn disable(&self) {
        *self.is_enabled.lock().unwrap() = false;
        *self.server_url.lock().unwrap() = None;
        *self.access_token.lock().unwrap() = None;
        *self.device_id.lock().unwrap() = None;
        *self.status.lock().unwrap() = SyncStatus::default();
    }

    pub fn get_config(&self) -> Option<SyncConfig> {
        let is_enabled = *self.is_enabled.lock().unwrap();
        if !is_enabled {
            return None;
        }

        let server_url = self.server_url.lock().unwrap().clone()?;
        let access_token = self.access_token.lock().unwrap().clone()?;
        let device_id = self.device_id.lock().unwrap().clone()?;

        Some(SyncConfig {
            server_url,
            access_token,
            device_id,
        })
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

/// Sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub server_url: String,
    pub access_token: String,
    pub device_id: String,
}
