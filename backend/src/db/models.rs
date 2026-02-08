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

// Emergency Access Models

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyContactStatus {
    Pending,
    Accepted,
    Revoked,
}

impl From<String> for EmergencyContactStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => EmergencyContactStatus::Pending,
            "accepted" => EmergencyContactStatus::Accepted,
            "revoked" => EmergencyContactStatus::Revoked,
            _ => EmergencyContactStatus::Pending,
        }
    }
}

impl From<EmergencyContactStatus> for String {
    fn from(s: EmergencyContactStatus) -> Self {
        match s {
            EmergencyContactStatus::Pending => "pending".to_string(),
            EmergencyContactStatus::Accepted => "accepted".to_string(),
            EmergencyContactStatus::Revoked => "revoked".to_string(),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct EmergencyContactRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub contact_email: String,
    pub contact_name: Option<String>,
    pub contact_user_id: Option<Uuid>,
    pub status: String,
    pub waiting_period_hours: i32,
    pub can_view_vault: Option<bool>,
    pub invitation_token: Option<String>,
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub id: Uuid,
    pub user_id: Uuid,
    pub contact_email: String,
    pub contact_name: Option<String>,
    pub contact_user_id: Option<Uuid>,
    pub status: EmergencyContactStatus,
    pub waiting_period_hours: i32,
    pub can_view_vault: bool,
    pub invitation_token: Option<String>,
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<EmergencyContactRow> for EmergencyContact {
    fn from(row: EmergencyContactRow) -> Self {
        EmergencyContact {
            id: row.id,
            user_id: row.user_id,
            contact_email: row.contact_email,
            contact_name: row.contact_name,
            contact_user_id: row.contact_user_id,
            status: EmergencyContactStatus::from(row.status),
            waiting_period_hours: row.waiting_period_hours,
            can_view_vault: row.can_view_vault.unwrap_or(true),
            invitation_token: row.invitation_token,
            invitation_expires_at: row.invitation_expires_at,
            accepted_at: row.accepted_at,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyAccessRequestStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

impl From<String> for EmergencyAccessRequestStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => EmergencyAccessRequestStatus::Pending,
            "approved" => EmergencyAccessRequestStatus::Approved,
            "denied" => EmergencyAccessRequestStatus::Denied,
            "expired" => EmergencyAccessRequestStatus::Expired,
            _ => EmergencyAccessRequestStatus::Pending,
        }
    }
}

impl From<EmergencyAccessRequestStatus> for String {
    fn from(s: EmergencyAccessRequestStatus) -> Self {
        match s {
            EmergencyAccessRequestStatus::Pending => "pending".to_string(),
            EmergencyAccessRequestStatus::Approved => "approved".to_string(),
            EmergencyAccessRequestStatus::Denied => "denied".to_string(),
            EmergencyAccessRequestStatus::Expired => "expired".to_string(),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct EmergencyAccessRequestRow {
    pub id: Uuid,
    pub emergency_contact_id: Uuid,
    pub status: String,
    pub request_reason: Option<String>,
    pub waiting_period_ends_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub denied_at: Option<DateTime<Utc>>,
    pub vault_key_encrypted: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAccessRequest {
    pub id: Uuid,
    pub emergency_contact_id: Uuid,
    pub status: EmergencyAccessRequestStatus,
    pub request_reason: Option<String>,
    pub waiting_period_ends_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub denied_at: Option<DateTime<Utc>>,
    pub vault_key_encrypted: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<EmergencyAccessRequestRow> for EmergencyAccessRequest {
    fn from(row: EmergencyAccessRequestRow) -> Self {
        EmergencyAccessRequest {
            id: row.id,
            emergency_contact_id: row.emergency_contact_id,
            status: EmergencyAccessRequestStatus::from(row.status),
            request_reason: row.request_reason,
            waiting_period_ends_at: row.waiting_period_ends_at,
            approved_at: row.approved_at,
            denied_at: row.denied_at,
            vault_key_encrypted: row.vault_key_encrypted,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmergencyAccessLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub emergency_contact_id: Option<Uuid>,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Remote Command Models

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RemoteCommandType {
    Lock,
    Wipe,
}

impl From<String> for RemoteCommandType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "lock" => RemoteCommandType::Lock,
            "wipe" => RemoteCommandType::Wipe,
            _ => RemoteCommandType::Lock,
        }
    }
}

impl From<RemoteCommandType> for String {
    fn from(t: RemoteCommandType) -> Self {
        match t {
            RemoteCommandType::Lock => "lock".to_string(),
            RemoteCommandType::Wipe => "wipe".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RemoteCommandStatus {
    Pending,
    Delivered,
    Executed,
    Failed,
}

impl From<String> for RemoteCommandStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => RemoteCommandStatus::Pending,
            "delivered" => RemoteCommandStatus::Delivered,
            "executed" => RemoteCommandStatus::Executed,
            "failed" => RemoteCommandStatus::Failed,
            _ => RemoteCommandStatus::Pending,
        }
    }
}

impl From<RemoteCommandStatus> for String {
    fn from(s: RemoteCommandStatus) -> Self {
        match s {
            RemoteCommandStatus::Pending => "pending".to_string(),
            RemoteCommandStatus::Delivered => "delivered".to_string(),
            RemoteCommandStatus::Executed => "executed".to_string(),
            RemoteCommandStatus::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct RemoteCommandRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_device_id: Uuid,
    pub command_type: String,
    pub status: String,
    pub issued_by_device_id: Option<Uuid>,
    pub issued_by_emergency_contact_id: Option<Uuid>,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommand {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_device_id: Uuid,
    pub command_type: RemoteCommandType,
    pub status: RemoteCommandStatus,
    pub issued_by_device_id: Option<Uuid>,
    pub issued_by_emergency_contact_id: Option<Uuid>,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<RemoteCommandRow> for RemoteCommand {
    fn from(row: RemoteCommandRow) -> Self {
        RemoteCommand {
            id: row.id,
            user_id: row.user_id,
            target_device_id: row.target_device_id,
            command_type: RemoteCommandType::from(row.command_type),
            status: RemoteCommandStatus::from(row.status),
            issued_by_device_id: row.issued_by_device_id,
            issued_by_emergency_contact_id: row.issued_by_emergency_contact_id,
            executed_at: row.executed_at,
            created_at: row.created_at,
        }
    }
}
