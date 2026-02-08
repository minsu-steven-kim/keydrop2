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
async fn register_user(router: &axum::Router, email: &str) -> String {
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

    json["access_token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_add_emergency_contact() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let access_token = register_user(&router, &email).await;

    let req = auth_json_request(
        Method::POST,
        "/api/v1/emergency/contacts",
        json!({
            "email": "trusted@example.com",
            "name": "Trusted Person",
            "waiting_period_hours": 48
        }),
        &access_token,
    );

    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("id").is_some());
    assert_eq!(json["contact_email"], "trusted@example.com");
    assert_eq!(json["contact_name"], "Trusted Person");
    assert_eq!(json["status"], "pending");
    assert_eq!(json["waiting_period_hours"], 48);
}

#[tokio::test]
async fn test_list_emergency_contacts() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let access_token = register_user(&router, &email).await;

    // Add contacts
    let add_req1 = auth_json_request(
        Method::POST,
        "/api/v1/emergency/contacts",
        json!({
            "email": "contact1@example.com",
            "name": "Contact One"
        }),
        &access_token,
    );
    router.clone().oneshot(add_req1).await.unwrap();

    let add_req2 = auth_json_request(
        Method::POST,
        "/api/v1/emergency/contacts",
        json!({
            "email": "contact2@example.com",
            "name": "Contact Two"
        }),
        &access_token,
    );
    router.clone().oneshot(add_req2).await.unwrap();

    // List contacts
    let list_req = auth_request(Method::GET, "/api/v1/emergency/contacts", &access_token);
    let response = router.oneshot(list_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let contacts = json.as_array().unwrap();

    assert_eq!(contacts.len(), 2);
}

#[tokio::test]
async fn test_remove_emergency_contact() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let access_token = register_user(&router, &email).await;

    // Add contact
    let add_req = auth_json_request(
        Method::POST,
        "/api/v1/emergency/contacts",
        json!({
            "email": "toremove@example.com"
        }),
        &access_token,
    );
    let add_response = router.clone().oneshot(add_req).await.unwrap();
    let body = axum::body::to_bytes(add_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let contact_id = json["id"].as_str().unwrap();

    // Remove contact
    let remove_req = auth_request(
        Method::DELETE,
        &format!("/api/v1/emergency/contacts/{}", contact_id),
        &access_token,
    );
    let remove_response = router.clone().oneshot(remove_req).await.unwrap();
    assert_eq!(remove_response.status(), StatusCode::OK);

    // Verify removal
    let list_req = auth_request(Method::GET, "/api/v1/emergency/contacts", &access_token);
    let list_response = router.oneshot(list_req).await.unwrap();
    let body = axum::body::to_bytes(list_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let contacts = json.as_array().unwrap();

    assert!(contacts.is_empty());
}

#[tokio::test]
async fn test_remove_contact_not_found() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let access_token = register_user(&router, &email).await;

    let remove_req = auth_request(
        Method::DELETE,
        "/api/v1/emergency/contacts/00000000-0000-0000-0000-000000000000",
        &access_token,
    );
    let response = router.oneshot(remove_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_emergency_access_logs() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let access_token = register_user(&router, &email).await;

    // Add a contact (which creates a log entry)
    let add_req = auth_json_request(
        Method::POST,
        "/api/v1/emergency/contacts",
        json!({
            "email": "trusted@example.com"
        }),
        &access_token,
    );
    router.clone().oneshot(add_req).await.unwrap();

    // Get logs
    let logs_req = auth_request(Method::GET, "/api/v1/emergency/logs", &access_token);
    let response = router.oneshot(logs_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let logs = json.as_array().unwrap();

    assert!(!logs.is_empty());
    assert_eq!(logs[0]["action"], "contact_added");
}

#[tokio::test]
async fn test_list_pending_requests_empty() {
    let (router, _pool) = create_test_router().await;
    let email = random_email();
    let access_token = register_user(&router, &email).await;

    let req = auth_request(Method::GET, "/api/v1/emergency/requests", &access_token);
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let requests = json.as_array().unwrap();

    assert!(requests.is_empty());
}

#[tokio::test]
async fn test_emergency_request_flow() {
    let (router, _pool) = create_test_router().await;

    // Create vault owner
    let owner_email = random_email();
    let owner_token = register_user(&router, &owner_email).await;

    // Create emergency contact user
    let contact_email = random_email();
    let contact_token = register_user(&router, &contact_email).await;

    // Owner adds contact
    let add_req = auth_json_request(
        Method::POST,
        "/api/v1/emergency/contacts",
        json!({
            "email": contact_email,
            "waiting_period_hours": 1 // Short for testing
        }),
        &owner_token,
    );
    let add_response = router.clone().oneshot(add_req).await.unwrap();
    assert_eq!(add_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(add_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let contact_id = json["id"].as_str().unwrap();

    // Note: In a full test, we'd need to:
    // 1. Have the contact accept the invitation
    // 2. Have the contact request access
    // 3. Have the owner deny or wait for auto-approval

    // For now, verify the contact was created
    let list_req = auth_request(Method::GET, "/api/v1/emergency/contacts", &owner_token);
    let list_response = router.oneshot(list_req).await.unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(list_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let contacts = json.as_array().unwrap();

    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0]["id"], contact_id);
}
