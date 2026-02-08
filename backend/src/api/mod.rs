use axum::{routing::get, Router};

use crate::AppState;

pub mod auth;
pub mod devices;
pub mod emergency;
pub mod sync;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth::router())
        .nest("/sync", sync::router())
        .nest("/devices", devices::router())
        .nest("/emergency", emergency::router())
}

async fn health_check() -> &'static str {
    "OK"
}
