use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{auth::jwt, AppError, AppState, Result};

/// Authenticated user information extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub device_id: Uuid,
}

/// Extract bearer token from Authorization header
fn extract_bearer_token(req: &Request) -> Result<&str> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    if !header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Invalid authorization header".to_string()));
    }

    Ok(&header[7..])
}

/// Authentication middleware that validates JWT and extracts user info
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let token = extract_bearer_token(&req)?;
    let claims = jwt::validate_access_token(token, &state.jwt_secret)?;

    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidToken)?;

    let device_id = claims
        .device_id
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidToken)?;

    let auth_user = AuthUser { user_id, device_id };
    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

/// Extension trait to easily extract AuthUser from request extensions
pub trait AuthUserExt {
    fn auth_user(&self) -> Result<&AuthUser>;
}

impl<B> AuthUserExt for axum::extract::Request<B> {
    fn auth_user(&self) -> Result<&AuthUser> {
        self.extensions()
            .get::<AuthUser>()
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))
    }
}
