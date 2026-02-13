use std::collections::HashMap;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::Response,
    routing::{get, post},
    Json, Router,
};
use axum_extra::TypedHeader;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use headers::{authorization::Bearer, Authorization};
use serde::Deserialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    auth::{jwt::validate_access_token, AuthUser},
    blob::BlobStorage,
    db,
    sync::{
        resolve_conflict, ConflictResolution, ConflictStrategy, SyncItem, SyncNotification,
        SyncNotificationType, SyncPullResponse, SyncPushRequest, SyncPushResponse,
    },
    AppError, AppState, Result,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pull", get(pull))
        .route("/push", post(push))
        .route("/notify", get(notify_ws))
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

#[derive(Debug, Deserialize)]
pub struct PullQuery {
    pub since_version: Option<i64>,
    pub limit: Option<i64>,
}

async fn pull(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Query(query): Query<PullQuery>,
) -> Result<Json<SyncPullResponse>> {
    let auth_user = extract_auth(&state, auth_header).await?;
    let blob_storage = state
        .blob_storage
        .as_ref()
        .ok_or_else(|| AppError::Internal("Blob storage not configured".into()))?;
    let since_version = query.since_version.unwrap_or(0);
    let limit = query.limit.unwrap_or(100).min(1000) as usize;

    // Get current server version
    let current_version = db::get_sync_version(&state.db, auth_user.user_id).await?;

    // Get items changed since requested version
    let items =
        db::get_vault_items_since_version(&state.db, auth_user.user_id, since_version).await?;

    // Fetch encrypted data for each item
    let mut sync_items = Vec::new();
    let mut item_count = 0;

    for item in items {
        if item_count >= limit {
            break;
        }

        // Retrieve encrypted blob
        let encrypted_data = match blob_storage.retrieve(&item.encrypted_blob_id).await {
            Ok(data) => base64::engine::general_purpose::STANDARD.encode(&data),
            Err(e) => {
                tracing::warn!("Failed to retrieve blob {}: {}", item.encrypted_blob_id, e);
                continue;
            }
        };

        sync_items.push(SyncItem {
            id: item.id,
            encrypted_data,
            version: item.version,
            is_deleted: item.is_deleted,
            modified_at: item.modified_at.timestamp(),
        });

        item_count += 1;
    }

    let has_more = item_count >= limit;

    // Update device last seen
    db::update_device_last_seen(&state.db, auth_user.device_id).await?;

    Ok(Json(SyncPullResponse {
        current_version,
        items: sync_items,
        has_more,
    }))
}

async fn push(
    State(state): State<AppState>,
    auth_header: TypedHeader<Authorization<Bearer>>,
    Json(req): Json<SyncPushRequest>,
) -> Result<Json<SyncPushResponse>> {
    let auth_user = extract_auth(&state, auth_header).await?;
    let blob_storage = state
        .blob_storage
        .as_ref()
        .ok_or_else(|| AppError::Internal("Blob storage not configured".into()))?;
    let current_version = db::get_sync_version(&state.db, auth_user.user_id).await?;

    // Check for version mismatch (client is behind)
    if req.base_version < current_version {
        // Get items that changed since client's base version
        let server_items =
            db::get_vault_items_since_version(&state.db, auth_user.user_id, req.base_version)
                .await?;

        // Build map of server items for conflict detection
        let server_items_map: HashMap<Uuid, _> =
            server_items.into_iter().map(|i| (i.id, i)).collect();

        let mut conflicts = Vec::new();
        let mut items_to_update = Vec::new();

        for client_item in &req.items {
            if let Some(server_item) = server_items_map.get(&client_item.id) {
                // Conflict detected - use last-write-wins strategy
                let server_sync_item = SyncItem {
                    id: server_item.id,
                    encrypted_data: String::new(), // Not needed for comparison
                    version: server_item.version,
                    is_deleted: server_item.is_deleted,
                    modified_at: server_item.modified_at.timestamp(),
                };

                let resolution = resolve_conflict(
                    &server_sync_item,
                    client_item,
                    ConflictStrategy::LastWriteWins,
                );

                match resolution {
                    ConflictResolution::UseClient => {
                        items_to_update.push(client_item.clone());
                    }
                    ConflictResolution::UseServer => {
                        // Fetch the server's encrypted data for the conflict response
                        if let Ok(data) =
                            blob_storage.retrieve(&server_item.encrypted_blob_id).await
                        {
                            conflicts.push(SyncItem {
                                id: server_item.id,
                                encrypted_data: base64::engine::general_purpose::STANDARD
                                    .encode(&data),
                                version: server_item.version,
                                is_deleted: server_item.is_deleted,
                                modified_at: server_item.modified_at.timestamp(),
                            });
                        }
                    }
                }
            } else {
                // No conflict - new item or item not modified on server
                items_to_update.push(client_item.clone());
            }
        }

        // Process items that should be updated
        let mut new_version = current_version;
        for item in items_to_update {
            new_version = process_sync_item(&state, auth_user.user_id, &item).await?;
        }

        // Notify other devices
        if new_version > current_version {
            let _ = state.sync_tx.send(SyncNotification {
                user_id: auth_user.user_id,
                notification_type: SyncNotificationType::ChangesAvailable,
                version: new_version,
                source_device_id: Some(auth_user.device_id),
            });
        }

        return Ok(Json(SyncPushResponse {
            new_version,
            had_conflicts: !conflicts.is_empty(),
            conflicts,
        }));
    }

    // No version conflict - process all items
    let mut new_version = current_version;
    for item in &req.items {
        new_version = process_sync_item(&state, auth_user.user_id, item).await?;
    }

    // Notify other devices
    if new_version > current_version {
        let _ = state.sync_tx.send(SyncNotification {
            user_id: auth_user.user_id,
            notification_type: SyncNotificationType::ChangesAvailable,
            version: new_version,
            source_device_id: Some(auth_user.device_id),
        });
    }

    // Update device last seen
    db::update_device_last_seen(&state.db, auth_user.device_id).await?;

    Ok(Json(SyncPushResponse {
        new_version,
        had_conflicts: false,
        conflicts: Vec::new(),
    }))
}

async fn process_sync_item(state: &AppState, user_id: Uuid, item: &SyncItem) -> Result<i64> {
    let blob_storage = state
        .blob_storage
        .as_ref()
        .ok_or_else(|| AppError::Internal("Blob storage not configured".into()))?;

    // Decode and store encrypted blob
    let encrypted_data = base64::engine::general_purpose::STANDARD
        .decode(&item.encrypted_data)
        .map_err(|e| AppError::BadRequest(format!("Invalid base64 data: {}", e)))?;

    let blob_id = BlobStorage::generate_blob_id(user_id);
    blob_storage.store(&blob_id, &encrypted_data).await?;

    // Increment version
    let new_version = db::increment_sync_version(&state.db, user_id).await?;

    // Upsert vault item record
    db::upsert_vault_item(
        &state.db,
        item.id,
        user_id,
        new_version,
        &blob_id,
        item.is_deleted,
    )
    .await?;

    Ok(new_version)
}

async fn notify_ws(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| handle_notify_ws(socket, state))
}

async fn handle_notify_ws(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for authentication message
    let auth_user = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            // Expect: {"token": "..."}
            #[derive(Deserialize)]
            struct AuthMessage {
                token: String,
            }

            match serde_json::from_str::<AuthMessage>(&text) {
                Ok(auth_msg) => match validate_access_token(&auth_msg.token, &state.jwt_secret) {
                    Ok(claims) => {
                        let user_id = match claims.sub.parse::<Uuid>() {
                            Ok(id) => id,
                            Err(_) => {
                                let _ = sender.send(Message::Close(None)).await;
                                return;
                            }
                        };
                        let device_id = match claims.device_id.parse::<Uuid>() {
                            Ok(id) => id,
                            Err(_) => {
                                let _ = sender.send(Message::Close(None)).await;
                                return;
                            }
                        };
                        AuthUser { user_id, device_id }
                    }
                    Err(_) => {
                        let _ = sender.send(Message::Close(None)).await;
                        return;
                    }
                },
                Err(_) => {
                    let _ = sender.send(Message::Close(None)).await;
                    return;
                }
            }
        }
        _ => {
            return;
        }
    };

    // Subscribe to sync notifications
    let mut rx = state.sync_tx.subscribe();

    // Send connected acknowledgment
    let _ = sender
        .send(Message::Text(
            serde_json::json!({"status": "connected"}).to_string(),
        ))
        .await;

    // Listen for notifications and forward to client
    loop {
        tokio::select! {
            // Handle incoming messages (ping/pong, close)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
            // Forward sync notifications
            notification = rx.recv() => {
                match notification {
                    Ok(notif) => {
                        // Only forward notifications for this user
                        if notif.user_id == auth_user.user_id {
                            // Don't notify the device that made the change
                            if notif.source_device_id != Some(auth_user.device_id) {
                                let msg = serde_json::to_string(&notif).unwrap_or_default();
                                if sender.send(Message::Text(msg)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Missed some messages, continue
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }
}
