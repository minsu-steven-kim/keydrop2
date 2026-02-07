use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, routing::post, Json, Router};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::jwt::{
        generate_token_pair, hash_refresh_token, validate_refresh_token, REFRESH_TOKEN_EXPIRY_DAYS,
    },
    db::{self, DeviceType},
    AppError, AppState, Result,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub auth_key: String, // Base64-encoded auth_key from client
    pub salt: String,     // Base64-encoded salt for the client to store
    pub device_name: String,
    pub device_type: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>> {
    // Check if user already exists
    if db::get_user_by_email(&state.db, &req.email).await?.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    // Hash the auth_key using Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let auth_key_hash = argon2
        .hash_password(req.auth_key.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash auth key: {}", e)))?
        .to_string();

    // Create user
    let user = db::create_user(&state.db, &req.email, &auth_key_hash, &req.salt).await?;

    // Create device
    let device_type = DeviceType::from(req.device_type);
    let device = db::create_device(&state.db, user.id, &req.device_name, device_type, None).await?;

    // Generate tokens
    let tokens = generate_token_pair(user.id, device.id, &state.jwt_secret)?;

    // Store refresh token hash
    let token_hash = hash_refresh_token(&tokens.refresh_token);
    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
    db::create_refresh_token(&state.db, user.id, device.id, &token_hash, expires_at).await?;

    // Initialize sync version for user
    db::increment_sync_version(&state.db, user.id).await?;

    Ok(Json(RegisterResponse {
        user_id: user.id,
        device_id: device.id,
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub auth_key: String,
    pub device_name: String,
    pub device_type: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub salt: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    // Find user
    let user = db::get_user_by_email(&state.db, &req.email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // Verify auth_key
    let parsed_hash = PasswordHash::new(&user.auth_key_hash)
        .map_err(|_| AppError::Internal("Invalid stored hash".to_string()))?;

    Argon2::default()
        .verify_password(req.auth_key.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)?;

    // Create or find device
    let device_type = DeviceType::from(req.device_type);
    let device = db::create_device(&state.db, user.id, &req.device_name, device_type, None).await?;

    // Generate tokens
    let tokens = generate_token_pair(user.id, device.id, &state.jwt_secret)?;

    // Store refresh token hash
    let token_hash = hash_refresh_token(&tokens.refresh_token);
    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
    db::create_refresh_token(&state.db, user.id, device.id, &token_hash, expires_at).await?;

    Ok(Json(LoginResponse {
        user_id: user.id,
        device_id: device.id,
        salt: user.salt,
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>> {
    // Validate the refresh token JWT
    let claims = validate_refresh_token(&req.refresh_token, &state.jwt_secret)?;

    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidToken)?;

    let device_id = claims
        .device_id
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidToken)?;

    // Verify refresh token exists in database
    let token_hash = hash_refresh_token(&req.refresh_token);
    let stored_token = db::get_refresh_token_by_hash(&state.db, &token_hash)
        .await?
        .ok_or(AppError::InvalidToken)?;

    // Delete old refresh token
    db::delete_refresh_token(&state.db, stored_token.id).await?;

    // Generate new token pair
    let tokens = generate_token_pair(user_id, device_id, &state.jwt_secret)?;

    // Store new refresh token hash
    let new_token_hash = hash_refresh_token(&tokens.refresh_token);
    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
    db::create_refresh_token(&state.db, user_id, device_id, &new_token_hash, expires_at).await?;

    // Update device last seen
    db::update_device_last_seen(&state.db, device_id).await?;

    Ok(Json(RefreshResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
    }))
}
