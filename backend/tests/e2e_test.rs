mod common;

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use common::{create_test_router, random_email};

/// Helper to make JSON request
fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Helper to make authenticated request
fn auth_request(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap()
}

/// Helper to make authenticated JSON request
fn auth_json_request(method: Method, uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn test_full_user_journey() {
    let (router, _pool) = create_test_router().await;

    // 1. Register new user
    let email = random_email();
    let register_req = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "salt": "dGVzdF9zYWx0",
            "device_name": "Desktop",
            "device_type": "desktop"
        }),
    );

    let register_response = router.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(register_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let access_token = json["access_token"].as_str().unwrap().to_string();
    let _device_id1 = json["device_id"].as_str().unwrap().to_string();

    // 2. Create vault items
    let push_req = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": 1,
            "items": [
                {
                    "id": "item-login-1",
                    "encrypted_data": "ZW5jcnlwdGVkX2xvZ2lu",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067200
                },
                {
                    "id": "item-login-2",
                    "encrypted_data": "ZW5jcnlwdGVkX2xvZ2luXzI=",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067201
                }
            ]
        }),
        &access_token,
    );

    let push_response = router.clone().oneshot(push_req).await.unwrap();
    assert_eq!(push_response.status(), StatusCode::OK);

    // 3. Login on second device
    let login_req = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "device_name": "Mobile",
            "device_type": "android"
        }),
    );

    let login_response = router.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let access_token2 = json["access_token"].as_str().unwrap().to_string();

    // 4. Sync on second device
    let pull_req = auth_request(
        Method::GET,
        "/api/v1/sync/pull?since_version=0",
        &access_token2,
    );

    let pull_response = router.clone().oneshot(pull_req).await.unwrap();
    assert_eq!(pull_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(pull_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);

    // 5. List devices
    let devices_req = auth_request(Method::GET, "/api/v1/devices", &access_token);

    let devices_response = router.clone().oneshot(devices_req).await.unwrap();
    assert_eq!(devices_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(devices_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let devices = json.as_array().unwrap();
    assert_eq!(devices.len(), 2);

    // 6. Modify item on second device
    let push_req2 = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": 2,
            "items": [
                {
                    "id": "item-login-1",
                    "encrypted_data": "dXBkYXRlZF9sb2dpbg==",
                    "version": 2,
                    "is_deleted": false,
                    "modified_at": 1704067300
                }
            ]
        }),
        &access_token2,
    );

    let push_response2 = router.clone().oneshot(push_req2).await.unwrap();
    assert_eq!(push_response2.status(), StatusCode::OK);

    // 7. Sync on first device
    let pull_req2 = auth_request(
        Method::GET,
        "/api/v1/sync/pull?since_version=2",
        &access_token,
    );

    let pull_response2 = router.clone().oneshot(pull_req2).await.unwrap();
    assert_eq!(pull_response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(pull_response2.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "item-login-1");
}

#[tokio::test]
async fn test_remote_lock_wipe_flow() {
    let (router, _pool) = create_test_router().await;

    // Register user with device 1
    let email = random_email();
    let register_req = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "salt": "dGVzdF9zYWx0",
            "device_name": "Device 1",
            "device_type": "desktop"
        }),
    );

    let register_response = router.clone().oneshot(register_req).await.unwrap();
    let body = axum::body::to_bytes(register_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let access_token1 = json["access_token"].as_str().unwrap().to_string();

    // Login on device 2
    let login_req = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "device_name": "Device 2",
            "device_type": "android"
        }),
    );

    let login_response = router.clone().oneshot(login_req).await.unwrap();
    let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let access_token2 = json["access_token"].as_str().unwrap().to_string();
    let device_id2 = json["device_id"].as_str().unwrap();

    // Device 1 sends lock command to Device 2
    let lock_req = auth_json_request(
        Method::POST,
        &format!("/api/v1/devices/{}/lock", device_id2),
        json!({}),
        &access_token1,
    );

    let lock_response = router.clone().oneshot(lock_req).await.unwrap();
    assert_eq!(lock_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(lock_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert!(json.get("command_id").is_some());

    // Device 2 checks for commands
    let commands_req = auth_request(Method::GET, "/api/v1/devices/commands", &access_token2);

    let commands_response = router.clone().oneshot(commands_req).await.unwrap();
    assert_eq!(commands_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(commands_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let commands = json.as_array().unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0]["command_type"], "lock");

    // Device 2 acknowledges the command
    let command_id = commands[0]["id"].as_str().unwrap();
    let ack_req = auth_json_request(
        Method::POST,
        &format!("/api/v1/devices/commands/{}/ack", command_id),
        json!({"success": true}),
        &access_token2,
    );

    let ack_response = router.clone().oneshot(ack_req).await.unwrap();
    assert_eq!(ack_response.status(), StatusCode::OK);

    // Verify no more pending commands
    let commands_req2 = auth_request(Method::GET, "/api/v1/devices/commands", &access_token2);

    let commands_response2 = router.oneshot(commands_req2).await.unwrap();
    let body = axum::body::to_bytes(commands_response2.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let commands = json.as_array().unwrap();
    assert!(commands.is_empty());
}

#[tokio::test]
async fn test_cannot_lock_own_device() {
    let (router, _pool) = create_test_router().await;

    // Register user
    let email = random_email();
    let register_req = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "salt": "dGVzdF9zYWx0",
            "device_name": "Device 1",
            "device_type": "desktop"
        }),
    );

    let register_response = router.clone().oneshot(register_req).await.unwrap();
    let body = axum::body::to_bytes(register_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let access_token = json["access_token"].as_str().unwrap().to_string();
    let device_id = json["device_id"].as_str().unwrap();

    // Try to lock own device
    let lock_req = auth_json_request(
        Method::POST,
        &format!("/api/v1/devices/{}/lock", device_id),
        json!({}),
        &access_token,
    );

    let lock_response = router.oneshot(lock_req).await.unwrap();
    assert_eq!(lock_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_lock_other_users_device() {
    let (router, _pool) = create_test_router().await;

    // Register user 1
    let email1 = random_email();
    let register_req1 = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email1,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "salt": "dGVzdF9zYWx0",
            "device_name": "User 1 Device",
            "device_type": "desktop"
        }),
    );

    let register_response1 = router.clone().oneshot(register_req1).await.unwrap();
    let body = axum::body::to_bytes(register_response1.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let access_token1 = json["access_token"].as_str().unwrap().to_string();

    // Register user 2
    let email2 = random_email();
    let register_req2 = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email2,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "salt": "dGVzdF9zYWx0",
            "device_name": "User 2 Device",
            "device_type": "android"
        }),
    );

    let register_response2 = router.clone().oneshot(register_req2).await.unwrap();
    let body = axum::body::to_bytes(register_response2.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let device_id2 = json["device_id"].as_str().unwrap();

    // User 1 tries to lock user 2's device
    let lock_req = auth_json_request(
        Method::POST,
        &format!("/api/v1/devices/{}/lock", device_id2),
        json!({}),
        &access_token1,
    );

    let lock_response = router.oneshot(lock_req).await.unwrap();
    assert_eq!(lock_response.status(), StatusCode::NOT_FOUND);
}
