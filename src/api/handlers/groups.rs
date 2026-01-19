use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use chrono::Utc;

use crate::api::{AppState, response::success};
use crate::domain::models::*;
use crate::infrastructure::error::AppError;
use crate::middleware::auth::AuthUser;

pub async fn create_group(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(dto): Json<CreateGroupDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let group = Group {
        id: Uuid::new_v4(),
        name: dto.name,
        description: dto.description,
        created_by: user.id,
        created_at: Utc::now(),
    };

    let created_group = state.group_repo.create(&group).await?;
    
    // Automatically add creator as member
    state.group_repo.add_member(created_group.id, user.id).await?;
    
    success(created_group)
}

pub async fn list_groups(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let groups = state.group_repo.list_all().await?;
    success(groups)
}

pub async fn get_group(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let group = state
        .group_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Group with id {} not found", id)))?;
    success(group)
}

pub async fn update_group(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateGroupDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let group = state
        .group_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Group with id {} not found", id)))?;

    // Only group creator or admin can update
    if group.created_by != user.id && !user.is_admin {
        return Err(AppError::Forbidden(
            "Only group creator or admin can update group".to_string(),
        ));
    }

    let updated_group = state.group_repo.update(id, &dto).await?;
    success(updated_group)
}

pub async fn delete_group(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let group = state
        .group_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Group with id {} not found", id)))?;

    // Only group creator or admin can delete
    if group.created_by != user.id && !user.is_admin {
        return Err(AppError::Forbidden(
            "Only group creator or admin can delete group".to_string(),
        ));
    }

    state.group_repo.delete(id).await?;
    success(serde_json::json!({"message": "Group deleted successfully"}))
}

pub async fn get_members(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let members = state.group_repo.get_members(id).await?;
    success(members)
}

pub async fn add_member(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let group = state
        .group_repo
        .find_by_id(group_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Group with id {} not found", group_id)))?;

    // Only group creator or admin can add members
    if group.created_by != user.id && !user.is_admin {
        return Err(AppError::Forbidden(
            "Only group creator or admin can add members".to_string(),
        ));
    }

    state.group_repo.add_member(group_id, user_id).await?;
    success(serde_json::json!({"message": "Member added successfully"}))
}

pub async fn remove_member(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let group = state
        .group_repo
        .find_by_id(group_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Group with id {} not found", group_id)))?;

    // Admin can kick anyone, or group creator can kick
    if !user.is_admin && group.created_by != user.id {
        return Err(AppError::Forbidden(
            "Only admin or group creator can remove members".to_string(),
        ));
    }

    state.group_repo.remove_member(group_id, user_id).await?;
    success(serde_json::json!({"message": "Member removed successfully"}))
}

pub async fn get_stats(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = state.todo_repo.get_group_stats(id).await?;
    success(stats)
}
