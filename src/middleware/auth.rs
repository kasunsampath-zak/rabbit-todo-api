use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::domain::models::User;
use crate::infrastructure::error::AppError;
use crate::api::AppState;

pub struct AuthUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        // Check for Basic auth
        if !auth_header.starts_with("Basic ") {
            return Err(AppError::Unauthorized(
                "Invalid Authorization header format".to_string(),
            ));
        }

        // Decode base64
        let encoded = &auth_header[6..];
        let decoded = BASE64
            .decode(encoded)
            .map_err(|_| AppError::Unauthorized("Invalid base64 encoding".to_string()))?;

        let credentials = String::from_utf8(decoded)
            .map_err(|_| AppError::Unauthorized("Invalid UTF-8 in credentials".to_string()))?;

        // Split username:password
        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AppError::Unauthorized(
                "Invalid credentials format".to_string(),
            ));
        }

        let username = parts[0];
        let password = parts[1];

        // Find user by username
        let user = state
            .user_repo
            .find_by_username(username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        // Verify password
        let valid = bcrypt::verify(password, &user.password_hash)
            .map_err(|_| AppError::InternalError("Password verification failed".to_string()))?;

        if !valid {
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        Ok(AuthUser(user))
    }
}

pub struct AdminUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;

        if !user.is_admin {
            return Err(AppError::Forbidden(
                "Admin access required".to_string(),
            ));
        }

        Ok(AdminUser(user))
    }
}
