use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use axum_extra::TypedHeader;
use chrono::{Duration, Utc};
use headers::{authorization::Bearer, Authorization};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::jwt::validate_access_token,
    db::{self, EmergencyAccessRequestStatus, EmergencyContactStatus},
    sync::{SyncNotification, SyncNotificationType},
    AppError, AppState, Result,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/contacts", post(add_contact))
        .route("/contacts", get(list_contacts))
        .route("/contacts/{id}", delete(remove_contact))
        .route("/contacts/{id}/accept", post(accept_invitation))
        .route("/request", post(request_access))
        .route("/requests", get(list_requests))
        .route("/requests/{id}/deny", post(deny_request))
        .route("/vault", get(get_vault_access))
        .route("/granted", get(list_granted_access))
        .route("/logs", get(get_logs))
}

/// Extract user_id from Authorization header
async fn extract_user_id(
    state: &AppState,
    auth_header: &TypedHeader<Authorization<Bearer>>,
) -> Result<Uuid> {
    let token = auth_header.token();
    let claims = validate_access_token(token, &state.jwt_secret)?;
    claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidToken)
}

// ============ Contact Management ============

#[derive(Debug, Deserialize)]
pub struct AddContactRequest {
    pub email: String,
    pub name: Option<String>,
    pub waiting_period_hours: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct EmergencyContactResponse {
    pub id: Uuid,
    pub contact_email: String,
    pub contact_name: Option<String>,
    pub status: String,
    pub waiting_period_hours: i32,
    pub can_view_vault: bool,
    pub accepted_at: Option<i64>,
    pub created_at: i64,
}

async fn add_contact(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Json(req): Json<AddContactRequest>,
) -> Result<Json<EmergencyContactResponse>> {
    let user_id = extract_user_id(&state, &auth_header).await?;

    // Generate invitation token
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut token_bytes);
    let invitation_token = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        token_bytes,
    );

    // Token expires in 7 days
    let invitation_expires_at = Utc::now() + Duration::days(7);
    let waiting_period = req.waiting_period_hours.unwrap_or(48);

    let contact = db::create_emergency_contact(
        &state.db,
        user_id,
        &req.email,
        req.name.as_deref(),
        waiting_period,
        &invitation_token,
        invitation_expires_at,
    )
    .await?;

    // Log the action
    db::create_emergency_access_log(
        &state.db,
        user_id,
        Some(contact.id),
        "contact_added",
        Some(serde_json::json!({ "email": req.email })),
        None,
    )
    .await?;

    Ok(Json(EmergencyContactResponse {
        id: contact.id,
        contact_email: contact.contact_email,
        contact_name: contact.contact_name,
        status: String::from(contact.status),
        waiting_period_hours: contact.waiting_period_hours,
        can_view_vault: contact.can_view_vault,
        accepted_at: contact.accepted_at.map(|t| t.timestamp()),
        created_at: contact.created_at.timestamp(),
    }))
}

async fn list_contacts(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<EmergencyContactResponse>>> {
    let user_id = extract_user_id(&state, &auth_header).await?;
    let contacts = db::get_emergency_contacts_by_user(&state.db, user_id).await?;

    let response: Vec<EmergencyContactResponse> = contacts
        .into_iter()
        .map(|c| EmergencyContactResponse {
            id: c.id,
            contact_email: c.contact_email,
            contact_name: c.contact_name,
            status: String::from(c.status),
            waiting_period_hours: c.waiting_period_hours,
            can_view_vault: c.can_view_vault,
            accepted_at: c.accepted_at.map(|t| t.timestamp()),
            created_at: c.created_at.timestamp(),
        })
        .collect();

    Ok(Json(response))
}

async fn remove_contact(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(contact_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user_id = extract_user_id(&state, &auth_header).await?;

    let contact = db::get_emergency_contact_by_id(&state.db, contact_id)
        .await?
        .ok_or(AppError::NotFound("Emergency contact not found".to_string()))?;

    if contact.user_id != user_id {
        return Err(AppError::NotFound("Emergency contact not found".to_string()));
    }

    db::delete_emergency_contact(&state.db, contact_id).await?;

    // Log the action
    db::create_emergency_access_log(
        &state.db,
        user_id,
        Some(contact_id),
        "contact_removed",
        None,
        None,
    )
    .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ============ Invitation Acceptance (Contact Side) ============

#[derive(Debug, Deserialize)]
pub struct AcceptInvitationRequest {
    pub token: String,
}

async fn accept_invitation(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(contact_id): Path<Uuid>,
    Json(req): Json<AcceptInvitationRequest>,
) -> Result<Json<serde_json::Value>> {
    let accepting_user_id = extract_user_id(&state, &auth_header).await?;

    // Find contact by ID and verify token
    let contact = db::get_emergency_contact_by_id(&state.db, contact_id)
        .await?
        .ok_or(AppError::NotFound("Invitation not found".to_string()))?;

    // Verify token matches and hasn't expired
    if contact.invitation_token.as_deref() != Some(&req.token) {
        return Err(AppError::BadRequest("Invalid invitation token".to_string()));
    }

    if let Some(expires_at) = contact.invitation_expires_at {
        if expires_at < Utc::now() {
            return Err(AppError::BadRequest("Invitation has expired".to_string()));
        }
    }

    // Accept the invitation
    db::accept_emergency_contact_invitation(&state.db, contact_id, accepting_user_id).await?;

    // Log the action
    db::create_emergency_access_log(
        &state.db,
        contact.user_id,
        Some(contact_id),
        "invitation_accepted",
        Some(serde_json::json!({ "accepted_by_user_id": accepting_user_id.to_string() })),
        None,
    )
    .await?;

    // Notify the user who created the emergency contact
    let _ = state.sync_tx.send(SyncNotification {
        user_id: contact.user_id,
        notification_type: SyncNotificationType::EmergencyContactAccepted,
        version: 0,
        source_device_id: None,
    });

    Ok(Json(serde_json::json!({ "success": true })))
}

// ============ Access Request (Contact Side) ============

#[derive(Debug, Deserialize)]
pub struct RequestAccessRequest {
    pub emergency_contact_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccessRequestResponse {
    pub request_id: Uuid,
    pub status: String,
    pub waiting_period_ends_at: i64,
    pub created_at: i64,
}

async fn request_access(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Json(req): Json<RequestAccessRequest>,
) -> Result<Json<AccessRequestResponse>> {
    let requesting_user_id = extract_user_id(&state, &auth_header).await?;

    // Get the emergency contact and verify the requesting user is the contact
    let contact = db::get_emergency_contact_by_id(&state.db, req.emergency_contact_id)
        .await?
        .ok_or(AppError::NotFound("Emergency contact not found".to_string()))?;

    // Verify the requesting user is the contact
    if contact.contact_user_id != Some(requesting_user_id) {
        return Err(AppError::BadRequest(
            "You are not authorized for this emergency contact".to_string(),
        ));
    }

    // Verify contact is accepted
    if contact.status != EmergencyContactStatus::Accepted {
        return Err(AppError::BadRequest(
            "Emergency contact invitation has not been accepted".to_string(),
        ));
    }

    // Check for existing pending request
    let existing_requests = db::get_access_requests_by_contact(&state.db, contact.id).await?;
    if existing_requests
        .iter()
        .any(|r| r.status == EmergencyAccessRequestStatus::Pending)
    {
        return Err(AppError::BadRequest(
            "There is already a pending access request".to_string(),
        ));
    }

    // Calculate waiting period end
    let waiting_period_ends_at =
        Utc::now() + Duration::hours(contact.waiting_period_hours as i64);

    let access_request = db::create_emergency_access_request(
        &state.db,
        contact.id,
        req.reason.as_deref(),
        waiting_period_ends_at,
    )
    .await?;

    // Log the action
    db::create_emergency_access_log(
        &state.db,
        contact.user_id,
        Some(contact.id),
        "access_requested",
        Some(serde_json::json!({
            "request_id": access_request.id.to_string(),
            "reason": req.reason
        })),
        None,
    )
    .await?;

    // Notify the vault owner
    let _ = state.sync_tx.send(SyncNotification {
        user_id: contact.user_id,
        notification_type: SyncNotificationType::EmergencyAccessRequested,
        version: 0,
        source_device_id: None,
    });

    Ok(Json(AccessRequestResponse {
        request_id: access_request.id,
        status: String::from(access_request.status),
        waiting_period_ends_at: access_request.waiting_period_ends_at.timestamp(),
        created_at: access_request.created_at.timestamp(),
    }))
}

// ============ Request Management (User Side) ============

#[derive(Debug, Serialize)]
pub struct PendingAccessRequest {
    pub request_id: Uuid,
    pub contact_id: Uuid,
    pub contact_email: String,
    pub contact_name: Option<String>,
    pub reason: Option<String>,
    pub waiting_period_ends_at: i64,
    pub created_at: i64,
}

async fn list_requests(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<PendingAccessRequest>>> {
    let user_id = extract_user_id(&state, &auth_header).await?;

    let requests = db::get_pending_access_requests_for_user(&state.db, user_id).await?;
    let contacts = db::get_emergency_contacts_by_user(&state.db, user_id).await?;

    let response: Vec<PendingAccessRequest> = requests
        .into_iter()
        .filter_map(|r| {
            let contact = contacts.iter().find(|c| c.id == r.emergency_contact_id)?;
            Some(PendingAccessRequest {
                request_id: r.id,
                contact_id: contact.id,
                contact_email: contact.contact_email.clone(),
                contact_name: contact.contact_name.clone(),
                reason: r.request_reason,
                waiting_period_ends_at: r.waiting_period_ends_at.timestamp(),
                created_at: r.created_at.timestamp(),
            })
        })
        .collect();

    Ok(Json(response))
}

async fn deny_request(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Path(request_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let user_id = extract_user_id(&state, &auth_header).await?;

    // Get the request and verify ownership
    let request = db::get_emergency_access_request_by_id(&state.db, request_id)
        .await?
        .ok_or(AppError::NotFound("Access request not found".to_string()))?;

    let contact = db::get_emergency_contact_by_id(&state.db, request.emergency_contact_id)
        .await?
        .ok_or(AppError::NotFound("Emergency contact not found".to_string()))?;

    if contact.user_id != user_id {
        return Err(AppError::NotFound("Access request not found".to_string()));
    }

    if request.status != EmergencyAccessRequestStatus::Pending {
        return Err(AppError::BadRequest(
            "Request is not pending".to_string(),
        ));
    }

    db::deny_emergency_access_request(&state.db, request_id).await?;

    // Log the action
    db::create_emergency_access_log(
        &state.db,
        user_id,
        Some(contact.id),
        "access_denied",
        Some(serde_json::json!({ "request_id": request_id.to_string() })),
        None,
    )
    .await?;

    // Notify the contact
    if let Some(contact_user_id) = contact.contact_user_id {
        let _ = state.sync_tx.send(SyncNotification {
            user_id: contact_user_id,
            notification_type: SyncNotificationType::EmergencyAccessDenied,
            version: 0,
            source_device_id: None,
        });
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

// ============ Vault Access (Contact Side) ============

#[derive(Debug, Serialize)]
pub struct GrantedAccessInfo {
    pub contact_id: Uuid,
    pub user_email: String,
    pub request_id: Uuid,
    pub approved_at: i64,
    pub vault_key_encrypted: Option<String>,
}

async fn list_granted_access(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<GrantedAccessInfo>>> {
    let user_id = extract_user_id(&state, &auth_header).await?;

    // Get contacts where the current user is the contact_user_id
    let contacts = db::get_emergency_contacts_for_contact_user(&state.db, user_id).await?;

    let mut granted_access = Vec::new();

    for contact in contacts {
        let requests = db::get_access_requests_by_contact(&state.db, contact.id).await?;
        for request in requests {
            if request.status == EmergencyAccessRequestStatus::Approved {
                // Get the vault owner's email
                let user = db::get_user_by_id(&state.db, contact.user_id).await?;
                if let Some(user) = user {
                    granted_access.push(GrantedAccessInfo {
                        contact_id: contact.id,
                        user_email: user.email,
                        request_id: request.id,
                        approved_at: request.approved_at.map(|t| t.timestamp()).unwrap_or(0),
                        vault_key_encrypted: request.vault_key_encrypted,
                    });
                }
            }
        }
    }

    Ok(Json(granted_access))
}

async fn get_vault_access(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<serde_json::Value>> {
    let user_id = extract_user_id(&state, &auth_header).await?;

    // Get contacts where the current user is the contact_user_id
    let contacts = db::get_emergency_contacts_for_contact_user(&state.db, user_id).await?;

    // Auto-approve requests that have passed their waiting period
    for contact in &contacts {
        let requests = db::get_access_requests_by_contact(&state.db, contact.id).await?;
        for request in requests {
            if request.status == EmergencyAccessRequestStatus::Pending
                && request.waiting_period_ends_at <= Utc::now()
            {
                // Auto-approve (in real implementation, would encrypt vault key for contact)
                db::approve_emergency_access_request(&state.db, request.id, "").await?;

                // Log the auto-approval
                db::create_emergency_access_log(
                    &state.db,
                    contact.user_id,
                    Some(contact.id),
                    "access_auto_approved",
                    Some(serde_json::json!({ "request_id": request.id.to_string() })),
                    None,
                )
                .await?;

                // Notify the vault owner
                let _ = state.sync_tx.send(SyncNotification {
                    user_id: contact.user_id,
                    notification_type: SyncNotificationType::EmergencyAccessApproved,
                    version: 0,
                    source_device_id: None,
                });
            }
        }
    }

    // Return approved vault access info
    let granted = list_granted_access(State(state.clone()), auth_header).await?;

    Ok(Json(serde_json::json!({
        "granted_access": granted.0
    })))
}

// ============ Logs ============

#[derive(Debug, Serialize)]
pub struct AccessLogEntry {
    pub id: Uuid,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub created_at: i64,
}

async fn get_logs(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<AccessLogEntry>>> {
    let user_id = extract_user_id(&state, &auth_header).await?;

    let logs = db::get_emergency_access_logs_for_user(&state.db, user_id, 100).await?;

    let response: Vec<AccessLogEntry> = logs
        .into_iter()
        .map(|l| AccessLogEntry {
            id: l.id,
            action: l.action,
            details: l.details,
            created_at: l.created_at.timestamp(),
        })
        .collect();

    Ok(Json(response))
}
