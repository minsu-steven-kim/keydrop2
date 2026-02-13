use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use keydrop_backend::{api, blob, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "keydrop_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://keydrop:keydrop@localhost/keydrop".to_string());

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;

    tracing::info!("Database connected and migrations applied");

    // Initialize blob storage
    let blob_storage = Arc::new(blob::BlobStorage::new().await?);

    // Create broadcast channel for sync notifications (capacity 100)
    let (sync_tx, _) = broadcast::channel(100);

    // JWT secret
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "development-secret-change-me".to_string());

    let state = AppState {
        db,
        jwt_secret,
        blob_storage: Some(blob_storage),
        sync_tx,
    };

    // Build router
    let app = Router::new()
        .nest("/api/v1", api::router())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
