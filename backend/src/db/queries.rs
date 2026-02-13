use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;
use crate::Result;

// ============ User Queries ============

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    auth_key_hash: &str,
    salt: &str,
) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, auth_key_hash, salt, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(email)
    .bind(auth_key_hash)
    .bind(salt)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

// ============ Device Queries ============

pub async fn create_device(
    pool: &PgPool,
    user_id: Uuid,
    device_name: &str,
    device_type: DeviceType,
    public_key: Option<&str>,
) -> Result<Device> {
    let device_type_str: String = device_type.into();
    let row = sqlx::query_as::<_, DeviceRow>(
        r#"
        INSERT INTO devices (id, user_id, device_name, device_type, public_key, last_seen_at, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(device_name)
    .bind(device_type_str)
    .bind(public_key)
    .fetch_one(pool)
    .await?;

    Ok(Device::from(row))
}

pub async fn get_device_by_id(pool: &PgPool, device_id: Uuid) -> Result<Option<Device>> {
    let row = sqlx::query_as::<_, DeviceRow>(
        r#"
        SELECT * FROM devices WHERE id = $1
        "#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Device::from))
}

pub async fn get_devices_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Device>> {
    let rows = sqlx::query_as::<_, DeviceRow>(
        r#"
        SELECT * FROM devices WHERE user_id = $1 ORDER BY last_seen_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Device::from).collect())
}

pub async fn update_device_last_seen(pool: &PgPool, device_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE devices SET last_seen_at = NOW() WHERE id = $1
        "#,
    )
    .bind(device_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_device_push_token(
    pool: &PgPool,
    device_id: Uuid,
    push_token: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE devices SET push_token = $2 WHERE id = $1
        "#,
    )
    .bind(device_id)
    .bind(push_token)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_device(pool: &PgPool, device_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM devices WHERE id = $1
        "#,
    )
    .bind(device_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Vault Sync Queries ============

pub async fn get_sync_version(pool: &PgPool, user_id: Uuid) -> Result<i64> {
    let result = sqlx::query_as::<_, SyncVersion>(
        r#"
        SELECT * FROM sync_versions WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|sv| sv.current_version).unwrap_or(0))
}

pub async fn increment_sync_version(pool: &PgPool, user_id: Uuid) -> Result<i64> {
    let result = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO sync_versions (user_id, current_version, updated_at)
        VALUES ($1, 1, NOW())
        ON CONFLICT (user_id)
        DO UPDATE SET current_version = sync_versions.current_version + 1, updated_at = NOW()
        RETURNING current_version
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn get_vault_items_since_version(
    pool: &PgPool,
    user_id: Uuid,
    since_version: i64,
) -> Result<Vec<VaultItemSync>> {
    let items = sqlx::query_as::<_, VaultItemSync>(
        r#"
        SELECT * FROM vault_items_sync
        WHERE user_id = $1 AND version > $2
        ORDER BY version ASC
        "#,
    )
    .bind(user_id)
    .bind(since_version)
    .fetch_all(pool)
    .await?;

    Ok(items)
}

pub async fn upsert_vault_item(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    version: i64,
    encrypted_blob_id: &str,
    is_deleted: bool,
) -> Result<VaultItemSync> {
    let item = sqlx::query_as::<_, VaultItemSync>(
        r#"
        INSERT INTO vault_items_sync (id, user_id, version, encrypted_blob_id, modified_at, is_deleted, created_at)
        VALUES ($1, $2, $3, $4, NOW(), $5, NOW())
        ON CONFLICT (id)
        DO UPDATE SET
            version = $3,
            encrypted_blob_id = $4,
            modified_at = NOW(),
            is_deleted = $5
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(version)
    .bind(encrypted_blob_id)
    .bind(is_deleted)
    .fetch_one(pool)
    .await?;

    Ok(item)
}

pub async fn get_vault_item_by_id(
    pool: &PgPool,
    item_id: Uuid,
    user_id: Uuid,
) -> Result<Option<VaultItemSync>> {
    let item = sqlx::query_as::<_, VaultItemSync>(
        r#"
        SELECT * FROM vault_items_sync WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(item_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(item)
}

// ============ Refresh Token Queries ============

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    device_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken> {
    let token = sqlx::query_as::<_, RefreshToken>(
        r#"
        INSERT INTO refresh_tokens (id, user_id, device_id, token_hash, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(device_id)
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(token)
}

pub async fn get_refresh_token_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<RefreshToken>> {
    let token = sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT * FROM refresh_tokens WHERE token_hash = $1 AND expires_at > NOW()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(token)
}

pub async fn delete_refresh_token(pool: &PgPool, token_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM refresh_tokens WHERE id = $1
        "#,
    )
    .bind(token_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_expired_refresh_tokens(pool: &PgPool) -> Result<u64> {
    let result = sqlx::query(
        r#"
        DELETE FROM refresh_tokens WHERE expires_at <= NOW()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============ Auth Request Queries ============

pub async fn create_auth_request(
    pool: &PgPool,
    requester_device_id: Uuid,
    target_device_id: Uuid,
    challenge: &str,
    expires_at: DateTime<Utc>,
) -> Result<AuthRequest> {
    let request = sqlx::query_as::<_, AuthRequest>(
        r#"
        INSERT INTO auth_requests (id, requester_device_id, target_device_id, challenge, status, expires_at, created_at)
        VALUES ($1, $2, $3, $4, 'pending', $5, NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(requester_device_id)
    .bind(target_device_id)
    .bind(challenge)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(request)
}

pub async fn get_auth_request_by_id(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Option<AuthRequest>> {
    let request = sqlx::query_as::<_, AuthRequest>(
        r#"
        SELECT * FROM auth_requests WHERE id = $1
        "#,
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await?;

    Ok(request)
}

pub async fn get_pending_auth_requests_for_device(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<Vec<AuthRequest>> {
    let requests = sqlx::query_as::<_, AuthRequest>(
        r#"
        SELECT * FROM auth_requests
        WHERE target_device_id = $1 AND status = 'pending' AND expires_at > NOW()
        ORDER BY created_at DESC
        "#,
    )
    .bind(device_id)
    .fetch_all(pool)
    .await?;

    Ok(requests)
}

pub async fn update_auth_request_response(
    pool: &PgPool,
    request_id: Uuid,
    response: &str,
    status: AuthRequestStatus,
) -> Result<()> {
    let status_str: String = status.into();
    sqlx::query(
        r#"
        UPDATE auth_requests SET response = $2, status = $3 WHERE id = $1
        "#,
    )
    .bind(request_id)
    .bind(response)
    .bind(status_str)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Emergency Contact Queries ============

pub async fn create_emergency_contact(
    pool: &PgPool,
    user_id: Uuid,
    contact_email: &str,
    contact_name: Option<&str>,
    waiting_period_hours: i32,
    invitation_token: &str,
    invitation_expires_at: DateTime<Utc>,
) -> Result<EmergencyContact> {
    let row = sqlx::query_as::<_, EmergencyContactRow>(
        r#"
        INSERT INTO emergency_contacts (user_id, contact_email, contact_name, waiting_period_hours, invitation_token, invitation_expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(contact_email)
    .bind(contact_name)
    .bind(waiting_period_hours)
    .bind(invitation_token)
    .bind(invitation_expires_at)
    .fetch_one(pool)
    .await?;

    Ok(EmergencyContact::from(row))
}

pub async fn get_emergency_contact_by_id(
    pool: &PgPool,
    contact_id: Uuid,
) -> Result<Option<EmergencyContact>> {
    let row = sqlx::query_as::<_, EmergencyContactRow>(
        r#"
        SELECT * FROM emergency_contacts WHERE id = $1
        "#,
    )
    .bind(contact_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(EmergencyContact::from))
}

pub async fn get_emergency_contacts_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<EmergencyContact>> {
    let rows = sqlx::query_as::<_, EmergencyContactRow>(
        r#"
        SELECT * FROM emergency_contacts WHERE user_id = $1 ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(EmergencyContact::from).collect())
}

pub async fn get_emergency_contacts_for_contact_user(
    pool: &PgPool,
    contact_user_id: Uuid,
) -> Result<Vec<EmergencyContact>> {
    let rows = sqlx::query_as::<_, EmergencyContactRow>(
        r#"
        SELECT * FROM emergency_contacts WHERE contact_user_id = $1 ORDER BY created_at DESC
        "#,
    )
    .bind(contact_user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(EmergencyContact::from).collect())
}

pub async fn get_emergency_contact_by_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<EmergencyContact>> {
    let row = sqlx::query_as::<_, EmergencyContactRow>(
        r#"
        SELECT * FROM emergency_contacts WHERE invitation_token = $1 AND invitation_expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(EmergencyContact::from))
}

pub async fn accept_emergency_contact_invitation(
    pool: &PgPool,
    contact_id: Uuid,
    contact_user_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE emergency_contacts
        SET status = 'accepted', contact_user_id = $2, accepted_at = NOW(), invitation_token = NULL
        WHERE id = $1
        "#,
    )
    .bind(contact_id)
    .bind(contact_user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_emergency_contact(pool: &PgPool, contact_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE emergency_contacts SET status = 'revoked' WHERE id = $1
        "#,
    )
    .bind(contact_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_emergency_contact(pool: &PgPool, contact_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        DELETE FROM emergency_contacts WHERE id = $1
        "#,
    )
    .bind(contact_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Emergency Access Request Queries ============

pub async fn create_emergency_access_request(
    pool: &PgPool,
    emergency_contact_id: Uuid,
    request_reason: Option<&str>,
    waiting_period_ends_at: DateTime<Utc>,
) -> Result<EmergencyAccessRequest> {
    let row = sqlx::query_as::<_, EmergencyAccessRequestRow>(
        r#"
        INSERT INTO emergency_access_requests (emergency_contact_id, request_reason, waiting_period_ends_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(emergency_contact_id)
    .bind(request_reason)
    .bind(waiting_period_ends_at)
    .fetch_one(pool)
    .await?;

    Ok(EmergencyAccessRequest::from(row))
}

pub async fn get_emergency_access_request_by_id(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Option<EmergencyAccessRequest>> {
    let row = sqlx::query_as::<_, EmergencyAccessRequestRow>(
        r#"
        SELECT * FROM emergency_access_requests WHERE id = $1
        "#,
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(EmergencyAccessRequest::from))
}

pub async fn get_pending_access_requests_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<EmergencyAccessRequest>> {
    let rows = sqlx::query_as::<_, EmergencyAccessRequestRow>(
        r#"
        SELECT ear.* FROM emergency_access_requests ear
        JOIN emergency_contacts ec ON ear.emergency_contact_id = ec.id
        WHERE ec.user_id = $1 AND ear.status = 'pending'
        ORDER BY ear.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(EmergencyAccessRequest::from).collect())
}

pub async fn get_access_requests_by_contact(
    pool: &PgPool,
    emergency_contact_id: Uuid,
) -> Result<Vec<EmergencyAccessRequest>> {
    let rows = sqlx::query_as::<_, EmergencyAccessRequestRow>(
        r#"
        SELECT * FROM emergency_access_requests WHERE emergency_contact_id = $1 ORDER BY created_at DESC
        "#,
    )
    .bind(emergency_contact_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(EmergencyAccessRequest::from).collect())
}

pub async fn deny_emergency_access_request(pool: &PgPool, request_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE emergency_access_requests SET status = 'denied', denied_at = NOW() WHERE id = $1
        "#,
    )
    .bind(request_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn approve_emergency_access_request(
    pool: &PgPool,
    request_id: Uuid,
    vault_key_encrypted: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE emergency_access_requests
        SET status = 'approved', approved_at = NOW(), vault_key_encrypted = $2
        WHERE id = $1
        "#,
    )
    .bind(request_id)
    .bind(vault_key_encrypted)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn expire_pending_access_requests(pool: &PgPool) -> Result<u64> {
    let result = sqlx::query(
        r#"
        UPDATE emergency_access_requests
        SET status = 'expired'
        WHERE status = 'pending' AND waiting_period_ends_at <= NOW()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============ Emergency Access Log Queries ============

pub async fn create_emergency_access_log(
    pool: &PgPool,
    user_id: Uuid,
    emergency_contact_id: Option<Uuid>,
    action: &str,
    details: Option<serde_json::Value>,
    ip_address: Option<&str>,
) -> Result<EmergencyAccessLog> {
    let log = sqlx::query_as::<_, EmergencyAccessLog>(
        r#"
        INSERT INTO emergency_access_logs (user_id, emergency_contact_id, action, details, ip_address)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(emergency_contact_id)
    .bind(action)
    .bind(details)
    .bind(ip_address)
    .fetch_one(pool)
    .await?;

    Ok(log)
}

pub async fn get_emergency_access_logs_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<EmergencyAccessLog>> {
    let logs = sqlx::query_as::<_, EmergencyAccessLog>(
        r#"
        SELECT * FROM emergency_access_logs WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

// ============ Remote Command Queries ============

pub async fn create_remote_command(
    pool: &PgPool,
    user_id: Uuid,
    target_device_id: Uuid,
    command_type: RemoteCommandType,
    issued_by_device_id: Option<Uuid>,
    issued_by_emergency_contact_id: Option<Uuid>,
) -> Result<RemoteCommand> {
    let command_type_str: String = command_type.into();
    let row = sqlx::query_as::<_, RemoteCommandRow>(
        r#"
        INSERT INTO remote_commands (user_id, target_device_id, command_type, issued_by_device_id, issued_by_emergency_contact_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(target_device_id)
    .bind(command_type_str)
    .bind(issued_by_device_id)
    .bind(issued_by_emergency_contact_id)
    .fetch_one(pool)
    .await?;

    Ok(RemoteCommand::from(row))
}

pub async fn get_pending_commands_for_device(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<Vec<RemoteCommand>> {
    let rows = sqlx::query_as::<_, RemoteCommandRow>(
        r#"
        SELECT * FROM remote_commands
        WHERE target_device_id = $1 AND status = 'pending'
        ORDER BY created_at ASC
        "#,
    )
    .bind(device_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(RemoteCommand::from).collect())
}

pub async fn update_command_status(
    pool: &PgPool,
    command_id: Uuid,
    status: RemoteCommandStatus,
) -> Result<()> {
    let status_str: String = status.into();
    let executed_at = if status == RemoteCommandStatus::Executed {
        Some(Utc::now())
    } else {
        None
    };

    sqlx::query(
        r#"
        UPDATE remote_commands SET status = $2, executed_at = $3 WHERE id = $1
        "#,
    )
    .bind(command_id)
    .bind(status_str)
    .bind(executed_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_commands_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<RemoteCommand>> {
    let rows = sqlx::query_as::<_, RemoteCommandRow>(
        r#"
        SELECT * FROM remote_commands WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(RemoteCommand::from).collect())
}
