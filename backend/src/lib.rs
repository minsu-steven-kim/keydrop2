//! Keydrop Backend Library
//!
//! Zero-knowledge sync backend for the Keydrop password manager.

pub mod api;
pub mod auth;
pub mod blob;
pub mod db;
pub mod error;
pub mod sync;

pub use error::{AppError, Result};

use std::sync::Arc;
use tokio::sync::broadcast;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt_secret: String,
    pub blob_storage: Arc<blob::BlobStorage>,
    /// Broadcast channel for real-time sync notifications
    pub sync_tx: broadcast::Sender<sync::SyncNotification>,
}
