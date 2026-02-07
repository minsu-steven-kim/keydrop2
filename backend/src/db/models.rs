use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub auth_key_hash: String,
    pub salt: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Android,
    Ios,
    Browser,
}

impl From<String> for DeviceType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "desktop" => DeviceType::Desktop,
            "android" => DeviceType::Android,
            "ios" => DeviceType::Ios,
            "browser" => DeviceType::Browser,
            _ => DeviceType::Desktop,
        }
    }
}

impl From<DeviceType> for String {
    fn from(dt: DeviceType) -> Self {
        match dt {
            DeviceType::Desktop => "desktop".to_string(),
            DeviceType::Android => "android".to_string(),
            DeviceType::Ios => "ios".to_string(),
            DeviceType::Browser => "browser".to_string(),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DeviceRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub public_key: Option<String>,
    pub push_token: Option<String>,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub public_key: Option<String>,
    pub push_token: Option<String>,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<DeviceRow> for Device {
    fn from(row: DeviceRow) -> Self {
        Device {
            id: row.id,
            user_id: row.user_id,
            device_name: row.device_name,
            device_type: DeviceType::from(row.device_type),
            public_key: row.public_key,
            push_token: row.push_token,
            last_seen_at: row.last_seen_at,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VaultItemSync {
    pub id: Uuid,
    pub user_id: Uuid,
    pub version: i64,
    pub encrypted_blob_id: String,
    pub modified_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SyncVersion {
    pub user_id: Uuid,
    pub current_version: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuthRequest {
    pub id: Uuid,
    pub requester_device_id: Uuid,
    pub target_device_id: Uuid,
    pub challenge: String,
    pub response: Option<String>,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthRequestStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

impl From<String> for AuthRequestStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => AuthRequestStatus::Pending,
            "approved" => AuthRequestStatus::Approved,
            "rejected" => AuthRequestStatus::Rejected,
            "expired" => AuthRequestStatus::Expired,
            _ => AuthRequestStatus::Pending,
        }
    }
}

impl From<AuthRequestStatus> for String {
    fn from(s: AuthRequestStatus) -> Self {
        match s {
            AuthRequestStatus::Pending => "pending".to_string(),
            AuthRequestStatus::Approved => "approved".to_string(),
            AuthRequestStatus::Rejected => "rejected".to_string(),
            AuthRequestStatus::Expired => "expired".to_string(),
        }
    }
}
