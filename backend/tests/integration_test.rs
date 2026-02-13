//! Integration tests for keydrop backend
//!
//! These tests verify the core functionality without requiring external services.

use keydrop_backend::auth::jwt::{
    generate_token_pair, validate_access_token, validate_refresh_token,
};
use keydrop_backend::sync::{resolve_conflict, ConflictResolution, ConflictStrategy, SyncItem};
use uuid::Uuid;

#[test]
fn test_jwt_generation_and_validation() {
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let secret = "test-secret-key";

    // Generate token pair
    let tokens = generate_token_pair(user_id, device_id, secret).unwrap();

    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());
    assert_eq!(tokens.expires_in, 15 * 60); // 15 minutes in seconds

    // Validate access token
    let claims = validate_access_token(&tokens.access_token, secret).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.device_id, device_id.to_string());

    // Validate refresh token
    let refresh_claims = validate_refresh_token(&tokens.refresh_token, secret).unwrap();
    assert_eq!(refresh_claims.sub, user_id.to_string());
}

#[test]
fn test_jwt_invalid_token() {
    let secret = "test-secret-key";
    let result = validate_access_token("invalid-token", secret);
    assert!(result.is_err());
}

#[test]
fn test_jwt_wrong_secret() {
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();

    let tokens = generate_token_pair(user_id, device_id, "correct-secret").unwrap();
    let result = validate_access_token(&tokens.access_token, "wrong-secret");
    assert!(result.is_err());
}

#[test]
fn test_conflict_resolution_last_write_wins() {
    let id = Uuid::new_v4();

    let server_item = SyncItem {
        id,
        encrypted_data: "server-data".to_string(),
        version: 1,
        is_deleted: false,
        modified_at: 1000,
    };

    let client_item = SyncItem {
        id,
        encrypted_data: "client-data".to_string(),
        version: 1,
        is_deleted: false,
        modified_at: 2000, // Client is newer
    };

    let result = resolve_conflict(&server_item, &client_item, ConflictStrategy::LastWriteWins);
    assert_eq!(result, ConflictResolution::UseClient);

    // Test when server is newer
    let server_newer = SyncItem {
        modified_at: 3000,
        ..server_item.clone()
    };

    let result = resolve_conflict(&server_newer, &client_item, ConflictStrategy::LastWriteWins);
    assert_eq!(result, ConflictResolution::UseServer);
}

#[test]
fn test_sync_item_serialization() {
    let item = SyncItem {
        id: Uuid::new_v4(),
        encrypted_data: "test-encrypted-data".to_string(),
        version: 42,
        is_deleted: false,
        modified_at: 1234567890,
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: SyncItem = serde_json::from_str(&json).unwrap();

    assert_eq!(item.id, deserialized.id);
    assert_eq!(item.version, deserialized.version);
    assert_eq!(item.encrypted_data, deserialized.encrypted_data);
}
