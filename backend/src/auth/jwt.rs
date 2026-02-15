use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, Result};

/// Access token validity (15 minutes)
const ACCESS_TOKEN_EXPIRY_MINUTES: i64 = 15;

/// Refresh token validity (30 days)
pub const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Device ID
    pub device_id: String,
    /// Expiration time (UTC timestamp)
    pub exp: i64,
    /// Issued at (UTC timestamp)
    pub iat: i64,
    /// Token type
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Generate an access token for a user
pub fn generate_access_token(user_id: Uuid, device_id: Uuid, secret: &str) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::minutes(ACCESS_TOKEN_EXPIRY_MINUTES);

    let claims = Claims {
        sub: user_id.to_string(),
        device_id: device_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        token_type: TokenType::Access,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))?;

    Ok(token)
}

/// Generate a refresh token for a user
pub fn generate_refresh_token(user_id: Uuid, device_id: Uuid, secret: &str) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);

    let claims = Claims {
        sub: user_id.to_string(),
        device_id: device_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        token_type: TokenType::Refresh,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))?;

    Ok(token)
}

/// Generate both access and refresh tokens
pub fn generate_token_pair(user_id: Uuid, device_id: Uuid, secret: &str) -> Result<TokenPair> {
    let access_token = generate_access_token(user_id, device_id, secret)?;
    let refresh_token = generate_refresh_token(user_id, device_id, secret)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        expires_in: ACCESS_TOKEN_EXPIRY_MINUTES * 60, // in seconds
    })
}

/// Validate and decode a token
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
        _ => AppError::InvalidToken,
    })?;

    Ok(token_data.claims)
}

/// Validate that a token is an access token
pub fn validate_access_token(token: &str, secret: &str) -> Result<Claims> {
    let claims = validate_token(token, secret)?;

    if claims.token_type != TokenType::Access {
        return Err(AppError::InvalidToken);
    }

    Ok(claims)
}

/// Validate that a token is a refresh token
pub fn validate_refresh_token(token: &str, secret: &str) -> Result<Claims> {
    let claims = validate_token(token, secret)?;

    if claims.token_type != TokenType::Refresh {
        return Err(AppError::InvalidToken);
    }

    Ok(claims)
}

/// Hash a refresh token for storage
pub fn hash_refresh_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        hasher.finalize(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2_hash_verify_cycle() {
        use argon2::{
            password_hash::{
                rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
            },
            Argon2,
        };

        let auth_key = "dGVzdF9hdXRoX2tleQ==";

        // Registration: hash the auth_key
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let auth_key_hash = argon2
            .hash_password(auth_key.as_bytes(), &salt)
            .expect("Failed to hash")
            .to_string();

        eprintln!("Hash: {}", auth_key_hash);
        eprintln!("Hash length: {}", auth_key_hash.len());

        // Login: parse and verify
        let parsed_hash = PasswordHash::new(&auth_key_hash).expect("Failed to parse hash");

        Argon2::default()
            .verify_password(auth_key.as_bytes(), &parsed_hash)
            .expect("Failed to verify");
    }

    #[test]
    fn test_token_generation() {
        let user_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let secret = "test_jwt_secret_key_for_testing_only";

        let tokens = generate_token_pair(user_id, device_id, secret).unwrap();
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert!(tokens.expires_in > 0);

        // Verify access token
        let claims = validate_access_token(&tokens.access_token, secret).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.device_id, device_id.to_string());

        // Verify refresh token
        let claims = validate_refresh_token(&tokens.refresh_token, secret).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.device_id, device_id.to_string());
    }
}
