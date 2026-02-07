use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod conflict;

pub use conflict::*;

/// Sync notification sent via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncNotification {
    /// User ID this notification is for
    pub user_id: Uuid,
    /// Type of notification
    pub notification_type: SyncNotificationType,
    /// New version number
    pub version: i64,
    /// Device that made the change (if applicable)
    pub source_device_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncNotificationType {
    /// New changes available for sync
    ChangesAvailable,
    /// Device was added
    DeviceAdded,
    /// Device was removed
    DeviceRemoved,
    /// Auth request pending
    AuthRequestPending,
    /// Auth request responded
    AuthRequestResponded,
}

/// Item change to be synced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncItem {
    /// Item ID
    pub id: Uuid,
    /// Encrypted blob (base64 encoded)
    pub encrypted_data: String,
    /// Item version
    pub version: i64,
    /// Whether the item is deleted
    pub is_deleted: bool,
    /// Modified timestamp (Unix timestamp)
    pub modified_at: i64,
}

/// Push request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPushRequest {
    /// Client's expected base version
    pub base_version: i64,
    /// Items to push
    pub items: Vec<SyncItem>,
}

/// Push response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPushResponse {
    /// New server version after push
    pub new_version: i64,
    /// Whether there were conflicts
    pub had_conflicts: bool,
    /// Conflicting items that need to be pulled
    pub conflicts: Vec<SyncItem>,
}

/// Pull response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPullResponse {
    /// Current server version
    pub current_version: i64,
    /// Items changed since requested version
    pub items: Vec<SyncItem>,
    /// Whether there are more items to pull
    pub has_more: bool,
}
