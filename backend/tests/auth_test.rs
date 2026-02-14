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

#[tokio::test]
async fn test_register_new_user() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();

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

    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("user_id").is_some());
    assert!(json.get("device_id").is_some());
    assert!(json.get("access_token").is_some());
    assert!(json.get("refresh_token").is_some());
    assert!(json.get("expires_in").is_some());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();

    // First registration
    let req1 = json_request(
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

    let response1 = router.clone().oneshot(req1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // Second registration with same email
    let req2 = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email,
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "salt": "dGVzdF9zYWx0",
            "device_name": "Test Device 2",
            "device_type": "android"
        }),
    );

    let response2 = router.oneshot(req2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login_valid_credentials() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let auth_key = "dGVzdF9hdXRoX2tleQ==";

    // Register first
    let register_req = json_request(
        Method::POST,
        "/api/v1/auth/register",
        json!({
            "email": email,
            "auth_key": auth_key,
            "salt": "dGVzdF9zYWx0",
            "device_name": "Test Device",
            "device_type": "desktop"
        }),
    );

    let register_response = router.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    // Login
    let login_req = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": email,
            "auth_key": auth_key,
            "device_name": "Test Device 2",
            "device_type": "android"
        }),
    );

    let login_response = router.oneshot(login_req).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(login_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("user_id").is_some());
    assert!(json.get("device_id").is_some());
    assert!(json.get("salt").is_some());
    assert!(json.get("access_token").is_some());
    assert!(json.get("refresh_token").is_some());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();

    // Register first
    let register_req = json_request(
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

    let register_response = router.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    // Login with wrong password
    let login_req = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": email,
            "auth_key": "d3JvbmdfYXV0aF9rZXk=",
            "device_name": "Test Device",
            "device_type": "desktop"
        }),
    );

    let login_response = router.oneshot(login_req).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let (router, _pool) = create_test_router().await;

    let login_req = json_request(
        Method::POST,
        "/api/v1/auth/login",
        json!({
            "email": "nonexistent@example.com",
            "auth_key": "dGVzdF9hdXRoX2tleQ==",
            "device_name": "Test Device",
            "device_type": "desktop"
        }),
    );

    let login_response = router.oneshot(login_req).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_token() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();

    // Register and get tokens
    let register_req = json_request(
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

    let register_response = router.clone().oneshot(register_req).await.unwrap();
    let body = axum::body::to_bytes(register_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let refresh_token = json["refresh_token"].as_str().unwrap();

    // Refresh
    let refresh_req = json_request(
        Method::POST,
        "/api/v1/auth/refresh",
        json!({
            "refresh_token": refresh_token
        }),
    );

    let refresh_response = router.oneshot(refresh_req).await.unwrap();
    assert_eq!(refresh_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(refresh_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("access_token").is_some());
    assert!(json.get("refresh_token").is_some());
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    let (router, _pool) = create_test_router().await;

    let refresh_req = json_request(
        Method::POST,
        "/api/v1/auth/refresh",
        json!({
            "refresh_token": "invalid_token"
        }),
    );

    let refresh_response = router.oneshot(refresh_req).await.unwrap();
    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_check() {
    let (router, _pool) = create_test_router().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
