use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use axum_extra::TypedHeader;
use base64::Engine;
use chrono::{Duration, Utc};
use headers::{authorization::Bearer, Authorization};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{jwt::validate_access_token, AuthUser},
    db::{self, AuthRequestStatus},
    sync::{SyncNotification, SyncNotificationType},
    AppError, AppState, Result,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_devices))
        .route("/{device_id}", get(get_device))
        .route("/{device_id}", delete(delete_device))
        .route("/{device_id}/push-token", post(update_push_token))
        .route("/{device_id}/auth-request", post(create_auth_request))
        .route("/{device_id}/auth-response", post(respond_auth_request))
        .route("/auth-requests/pending", get(get_pending_auth_requests))
}

/// Extract and validate auth from Authorization header
async fn extract_auth(
    state: &AppState,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<AuthUser> {
    let token = auth_header.token();
    let claims = validate_access_token(token, &state.jwt_secret)?;

    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidToken)?;

    let device_id = claims
        .device_id
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidToken)?;

    Ok(AuthUser { user_id, device_id })
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub last_seen_at: i64,
    pub created_at: i64,
    pub is_current: bool,
}

async fn list_devices(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<DeviceResponse>>> {
    let auth_user = extract_auth(&state, auth_header).await?;
    let devices = db::get_devices_by_user(&state.db, auth_user.user_id).await?;

    let response: Vec<DeviceResponse> = devices
        .into_iter()
        .map(|d| DeviceResponse {
            id: d.id,
            device_name: d.device_name,
            device_type: d.device_type.into(),
            last_seen_at: d.last_seen_at.timestamp(),
            created_at: d.created_at.timestamp(),
            is_current: d.id == auth_user.device_id,
        })
        .collect();

    Ok(Json(response))
}

async fn get_device(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<DeviceResponse>> {
    let auth_user = extract_auth(&state, auth_header).await?;
    let device = db::get_device_by_id(&state.db, device_id)
        .await?
        .ok_or(AppError::DeviceNotFound)?;

    // Verify device belongs to user
    if device.user_id != auth_user.user_id {
        return Err(AppError::DeviceNotFound);
    }

    Ok(Json(DeviceResponse {
        id: device.id,
        device_name: device.device_name,
        device_type: device.device_type.into(),
        last_seen_at: device.last_seen_at.timestamp(),
        created_at: device.created_at.timestamp(),
        is_current: device.id == auth_user.device_id,
    }))
}

async fn delete_device(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let auth_user = extract_auth(&state, auth_header).await?;
    let device = db::get_device_by_id(&state.db, device_id)
        .await?
        .ok_or(AppError::DeviceNotFound)?;

    // Verify device belongs to user
    if device.user_id != auth_user.user_id {
        return Err(AppError::DeviceNotFound);
    }

    // Can't delete current device
    if device.id == auth_user.device_id {
        return Err(AppError::BadRequest(
            "Cannot delete current device".to_string(),
        ));
    }

    db::delete_device(&state.db, device_id).await?;

    // Notify the deleted device
    let _ = state.sync_tx.send(SyncNotification {
        user_id: auth_user.user_id,
        notification_type: SyncNotificationType::DeviceRemoved,
        version: 0,
        source_device_id: Some(device_id),
    });

    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePushTokenRequest {
    pub push_token: String,
}

async fn update_push_token(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<UpdatePushTokenRequest>,
) -> Result<Json<serde_json::Value>> {
    let auth_user = extract_auth(&state, auth_header).await?;
    let device = db::get_device_by_id(&state.db, device_id)
        .await?
        .ok_or(AppError::DeviceNotFound)?;

    // Verify device belongs to user
    if device.user_id != auth_user.user_id {
        return Err(AppError::DeviceNotFound);
    }

    db::update_device_push_token(&state.db, device_id, &req.push_token).await?;

    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Debug, Serialize)]
pub struct AuthRequestResponse {
    pub request_id: Uuid,
    pub challenge: String,
    pub expires_at: i64,
}

async fn create_auth_request(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(target_device_id): Path<Uuid>,
) -> Result<Json<AuthRequestResponse>> {
    let auth_user = extract_auth(&state, auth_header).await?;

    // Verify target device belongs to user
    let target_device = db::get_device_by_id(&state.db, target_device_id)
        .await?
        .ok_or(AppError::DeviceNotFound)?;

    if target_device.user_id != auth_user.user_id {
        return Err(AppError::DeviceNotFound);
    }

    // Can't request auth from self
    if target_device_id == auth_user.device_id {
        return Err(AppError::BadRequest(
            "Cannot request auth from current device".to_string(),
        ));
    }

    // Generate challenge (random 32 bytes, base64 encoded)
    let mut challenge_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut challenge_bytes);
    let challenge = base64::engine::general_purpose::STANDARD.encode(challenge_bytes);

    // Expires in 5 minutes
    let expires_at = Utc::now() + Duration::minutes(5);

    let auth_request = db::create_auth_request(
        &state.db,
        auth_user.device_id,
        target_device_id,
        &challenge,
        expires_at,
    )
    .await?;

    // Notify target device
    let _ = state.sync_tx.send(SyncNotification {
        user_id: auth_user.user_id,
        notification_type: SyncNotificationType::AuthRequestPending,
        version: 0,
        source_device_id: Some(auth_user.device_id),
    });

    Ok(Json(AuthRequestResponse {
        request_id: auth_request.id,
        challenge,
        expires_at: expires_at.timestamp(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct AuthResponseRequest {
    pub request_id: Uuid,
    pub response: String, // Signed challenge
    pub approved: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseResponse {
    pub success: bool,
}

async fn respond_auth_request(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<AuthResponseRequest>,
) -> Result<Json<AuthResponseResponse>> {
    let auth_user = extract_auth(&state, auth_header).await?;

    // Verify this device is the target of the auth request
    let auth_request = db::get_auth_request_by_id(&state.db, req.request_id)
        .await?
        .ok_or(AppError::NotFound("Auth request not found".to_string()))?;

    if auth_request.target_device_id != device_id {
        return Err(AppError::BadRequest(
            "Device is not target of auth request".to_string(),
        ));
    }

    // Verify device belongs to user
    let device = db::get_device_by_id(&state.db, device_id)
        .await?
        .ok_or(AppError::DeviceNotFound)?;

    if device.user_id != auth_user.user_id {
        return Err(AppError::DeviceNotFound);
    }

    // Check if request is still pending
    if AuthRequestStatus::from(auth_request.status) != AuthRequestStatus::Pending {
        return Err(AppError::BadRequest(
            "Auth request is not pending".to_string(),
        ));
    }

    // Check if request has expired
    if auth_request.expires_at < Utc::now() {
        return Err(AppError::BadRequest("Auth request has expired".to_string()));
    }

    // Update the auth request
    let status = if req.approved {
        AuthRequestStatus::Approved
    } else {
        AuthRequestStatus::Rejected
    };

    db::update_auth_request_response(&state.db, req.request_id, &req.response, status).await?;

    // Notify requester device
    let _ = state.sync_tx.send(SyncNotification {
        user_id: auth_user.user_id,
        notification_type: SyncNotificationType::AuthRequestResponded,
        version: 0,
        source_device_id: Some(device_id),
    });

    Ok(Json(AuthResponseResponse { success: true }))
}

#[derive(Debug, Serialize)]
pub struct PendingAuthRequest {
    pub request_id: Uuid,
    pub requester_device_id: Uuid,
    pub challenge: String,
    pub expires_at: i64,
    pub created_at: i64,
}

async fn get_pending_auth_requests(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<PendingAuthRequest>>> {
    let auth_user = extract_auth(&state, auth_header).await?;
    let requests =
        db::get_pending_auth_requests_for_device(&state.db, auth_user.device_id).await?;

    let response: Vec<PendingAuthRequest> = requests
        .into_iter()
        .map(|r| PendingAuthRequest {
            request_id: r.id,
            requester_device_id: r.requester_device_id,
            challenge: r.challenge,
            expires_at: r.expires_at.timestamp(),
            created_at: r.created_at.timestamp(),
        })
        .collect();

    Ok(Json(response))
}
