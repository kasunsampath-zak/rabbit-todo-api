use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use chrono::Utc;

use crate::api::{AppState, response::success};
use crate::domain::models::*;
use crate::infrastructure::error::AppError;
use crate::middleware::auth::{AuthUser, AdminUser};

pub async fn create_user(
    State(state): State<AppState>,
    Json(dto): Json<CreateUserDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let password_hash = bcrypt::hash(&dto.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

    let user = User {
        id: Uuid::new_v4(),
        username: dto.username,
        password_hash,
        is_admin: dto.is_admin,
        points: 0,
        created_at: Utc::now(),
    };

    let created_user = state.user_repo.create(&user).await?;
    success(created_user)
}

pub async fn list_users(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let users = state.user_repo.list_all().await?;
    success(users)
}

pub async fn get_user(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = state
        .user_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;
    success(user)
}

pub async fn update_user(
    AuthUser(auth_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Users can only update their own profile unless they're admin
    if auth_user.id != id && !auth_user.is_admin {
        return Err(AppError::Forbidden(
            "You can only update your own profile".to_string(),
        ));
    }

    let updated_user = state.user_repo.update(id, &dto).await?;
    success(updated_user)
}

pub async fn delete_user(
    AdminUser(_admin): AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.user_repo.delete(id).await?;
    success(serde_json::json!({"message": "User deleted successfully"}))
}

pub async fn get_profile(
    AuthUser(auth_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Users can only view their own profile unless they're admin
    if auth_user.id != id && !auth_user.is_admin {
        return Err(AppError::Forbidden(
            "You can only view your own profile".to_string(),
        ));
    }

    let profile = state.user_repo.get_profile(id).await?;
    success(profile)
}
