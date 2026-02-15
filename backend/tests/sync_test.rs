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

/// Helper to register and get access token
async fn register_user(router: &axum::Router, email: &str) -> (String, String) {
    let req = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "salt": "dGVzdF9zYWx0",
            "device_name": "Test Device",
            "device_type": "desktop"
        }),
    );

    let response = router.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    (
        json["access_token"].as_str().unwrap().to_string(),
        json["device_id"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn test_pull_empty_vault() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let (access_token, _device_id) = register_user(&router, &email).await;

    let req = auth_request(
        Method::GET,
        "/api/v1/sync/pull?since_version=0",
        &access_token,
    );

    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("current_version").is_some());
    assert!(json.get("items").is_some());
    assert!(json["items"].as_array().unwrap().is_empty());
    assert_eq!(json.get("has_more"), Some(&json!(false)));
}

#[tokio::test]
async fn test_push_and_pull_items() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let (access_token, _device_id) = register_user(&router, &email).await;

    // Push an item
    let push_req = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": 1,
            "items": [
                {
                    "id": "10000000-0000-0000-0000-000000000001",
                    "encrypted_data": "ZW5jcnlwdGVkX2RhdGFfMQ==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067200
                }
            ]
        }),
        &access_token,
    );

    let push_response = router.clone().oneshot(push_req).await.unwrap();
    assert_eq!(push_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(push_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let new_version = json["new_version"].as_i64().unwrap();
    assert!(new_version > 1);

    // Pull to verify
    let pull_req = auth_request(
        Method::GET,
        "/api/v1/sync/pull?since_version=0",
        &access_token,
    );

    let pull_response = router.oneshot(pull_req).await.unwrap();
    assert_eq!(pull_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(pull_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "10000000-0000-0000-0000-000000000001");
}

#[tokio::test]
async fn test_push_multiple_items() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let (access_token, _device_id) = register_user(&router, &email).await;

    // Push multiple items
    let push_req = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": 1,
            "items": [
                {
                    "id": "10000000-0000-0000-0000-000000000001",
                    "encrypted_data": "ZW5jcnlwdGVkX2RhdGFfMQ==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067200
                },
                {
                    "id": "10000000-0000-0000-0000-000000000002",
                    "encrypted_data": "ZW5jcnlwdGVkX2RhdGFfMg==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067201
                },
                {
                    "id": "10000000-0000-0000-0000-000000000003",
                    "encrypted_data": "ZW5jcnlwdGVkX2RhdGFfMw==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067202
                }
            ]
        }),
        &access_token,
    );

    let push_response = router.clone().oneshot(push_req).await.unwrap();
    assert_eq!(push_response.status(), StatusCode::OK);

    // Pull to verify
    let pull_req = auth_request(
        Method::GET,
        "/api/v1/sync/pull?since_version=0",
        &access_token,
    );

    let pull_response = router.oneshot(pull_req).await.unwrap();
    assert_eq!(pull_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(pull_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
}

#[tokio::test]
async fn test_push_deleted_item() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let (access_token, _device_id) = register_user(&router, &email).await;

    // Push an item
    let push_req1 = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": 1,
            "items": [
                {
                    "id": "10000000-0000-0000-0000-000000000001",
                    "encrypted_data": "ZW5jcnlwdGVkX2RhdGFfMQ==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067200
                }
            ]
        }),
        &access_token,
    );

    let push_response1 = router.clone().oneshot(push_req1).await.unwrap();
    assert_eq!(push_response1.status(), StatusCode::OK);

    let body = axum::body::to_bytes(push_response1.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let version = json["new_version"].as_i64().unwrap();

    // Delete the item
    let push_req2 = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": version,
            "items": [
                {
                    "id": "10000000-0000-0000-0000-000000000001",
                    "encrypted_data": "",
                    "version": version,
                    "is_deleted": true,
                    "modified_at": 1704067300
                }
            ]
        }),
        &access_token,
    );

    let push_response2 = router.clone().oneshot(push_req2).await.unwrap();
    assert_eq!(push_response2.status(), StatusCode::OK);

    // Pull to verify
    let pull_req = auth_request(
        Method::GET,
        "/api/v1/sync/pull?since_version=0",
        &access_token,
    );

    let pull_response = router.oneshot(pull_req).await.unwrap();
    assert_eq!(pull_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(pull_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["is_deleted"], true);
}

#[tokio::test]
async fn test_pull_since_version() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let (access_token, _device_id) = register_user(&router, &email).await;

    // Push first item
    let push_req1 = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": 1,
            "items": [
                {
                    "id": "10000000-0000-0000-0000-000000000001",
                    "encrypted_data": "ZW5jcnlwdGVkX2RhdGFfMQ==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067200
                }
            ]
        }),
        &access_token,
    );

    let push_response1 = router.clone().oneshot(push_req1).await.unwrap();
    let body = axum::body::to_bytes(push_response1.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let version_after_first = json["new_version"].as_i64().unwrap();

    // Push second item
    let push_req2 = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": version_after_first,
            "items": [
                {
                    "id": "10000000-0000-0000-0000-000000000002",
                    "encrypted_data": "ZW5jcnlwdGVkX2RhdGFfMg==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067201
                }
            ]
        }),
        &access_token,
    );

    let push_response2 = router.clone().oneshot(push_req2).await.unwrap();
    assert_eq!(push_response2.status(), StatusCode::OK);

    // Pull since first version (should only get second item)
    let pull_req = auth_request(
        Method::GET,
        &format!("/api/v1/sync/pull?since_version={}", version_after_first),
        &access_token,
    );

    let pull_response = router.oneshot(pull_req).await.unwrap();
    assert_eq!(pull_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(pull_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "10000000-0000-0000-0000-000000000002");
}

#[tokio::test]
async fn test_sync_without_auth() {
    let (router, _pool) = create_test_router().await;

    // Try to pull without auth
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/sync/pull?since_version=0")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Missing auth header
}

#[tokio::test]
async fn test_two_devices_sync() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();

    // Register first device
    let (access_token1, _device_id1) = register_user(&router, &email).await;

    // Login on second device
    let login_req = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "device_name": "Second Device",
            "device_type": "android"
        }),
    );

    let login_response = router.clone().oneshot(login_req).await.unwrap();
    let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let access_token2 = json["access_token"].as_str().unwrap().to_string();

    // Push item from device 1
    let push_req = auth_json_request(
        Method::POST,
        "/api/v1/sync/push",
        json!({
            "base_version": 1,
            "items": [
                {
                    "id": "10000000-0000-0000-0000-000000000001",
                    "encrypted_data": "ZnJvbV9kZXZpY2VfMQ==",
                    "version": 0,
                    "is_deleted": false,
                    "modified_at": 1704067200
                }
            ]
        }),
        &access_token1,
    );

    let push_response = router.clone().oneshot(push_req).await.unwrap();
    assert_eq!(push_response.status(), StatusCode::OK);

    // Pull from device 2
    let pull_req = auth_request(
        Method::GET,
        "/api/v1/sync/pull?since_version=0",
        &access_token2,
    );

    let pull_response = router.oneshot(pull_req).await.unwrap();
    assert_eq!(pull_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(pull_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "10000000-0000-0000-0000-000000000001");
}
