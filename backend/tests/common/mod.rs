use axum::Router;
use keydrop_backend::{api, AppState};
use once_cell::sync::Lazy;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Test database URL (use a separate test database)
pub static TEST_DATABASE_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set")
});

/// Create a test database pool
pub async fn create_test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&TEST_DATABASE_URL)
        .await
        .expect("Failed to create test pool")
}

/// Run migrations on the test database
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}

/// Clean up test data (call before/after tests)
pub async fn cleanup_test_data(pool: &PgPool) {
    // Delete in order respecting foreign keys
    let tables = [
        "emergency_access_logs",
        "emergency_access_requests",
        "emergency_contacts",
        "remote_commands",
        "auth_requests",
        "refresh_tokens",
        "vault_items_sync",
        "sync_versions",
        "devices",
        "users",
    ];

    for table in tables {
        sqlx::query(&format!("DELETE FROM {}", table))
            .execute(pool)
            .await
            .ok();
    }
}

/// Create a test app state
pub async fn create_test_state(pool: PgPool) -> AppState {
    let (sync_tx, _) = broadcast::channel(100);

    AppState {
        db: pool,
        jwt_secret: "test_jwt_secret_key_for_testing_only".to_string(),
        sync_tx,
        blob_storage: None, // No blob storage in tests
    }
}

/// Create a test router
pub async fn create_test_router() -> (Router, PgPool) {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_test_data(&pool).await;

    let state = create_test_state(pool.clone()).await;
    let router = Router::new()
        .nest("/api/v1", api::router())
        .with_state(state);

    (router, pool)
}

/// Helper to generate a random email for testing
pub fn random_email() -> String {
    format!("test_{}@example.com", uuid::Uuid::new_v4())
}

/// Test user creation helper
pub struct TestUser {
    pub email: String,
    pub password: String,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl TestUser {
    pub fn new() -> Self {
        Self {
            email: random_email(),
            password: "test_password_123".to_string(),
            user_id: None,
            device_id: None,
            access_token: None,
            refresh_token: None,
        }
    }

    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token.as_ref().unwrap())
    }
}

impl Default for TestUser {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for test assertions
pub trait TestAssertions {
    fn assert_ok(&self);
    fn assert_status(&self, expected: u16);
}
