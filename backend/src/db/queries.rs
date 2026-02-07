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

pub async fn get_auth_request_by_id(pool: &PgPool, request_id: Uuid) -> Result<Option<AuthRequest>> {
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
