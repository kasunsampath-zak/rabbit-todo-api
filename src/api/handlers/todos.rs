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

pub async fn create_todo(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(dto): Json<CreateTodoDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify user is member of the group
    let is_member = state.group_repo.is_member(dto.group_id, user.id).await?;
    if !is_member && !user.is_admin {
        return Err(AppError::Forbidden(
            "You must be a member of the group to create todos".to_string(),
        ));
    }

    let todo = Todo {
        id: Uuid::new_v4(),
        title: dto.title,
        description: dto.description,
        status: TodoStatus::Active,
        priority: dto.priority,
        notes: dto.notes,
        estimated_duration: dto.estimated_duration,
        actual_duration: None,
        due_date: dto.due_date,
        group_id: dto.group_id,
        created_by: user.id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created_todo = state.todo_repo.create(&todo).await?;
    success(created_todo)
}

pub async fn get_todo(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let todo = state
        .todo_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    // Verify user is member of the group or admin
    let is_member = state.group_repo.is_member(todo.group_id, user.id).await?;
    if !is_member && !user.is_admin {
        return Err(AppError::Forbidden(
            "You must be a member of the group to view this todo".to_string(),
        ));
    }

    success(todo)
}

pub async fn update_todo(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateTodoDto>,
) -> Result<Json<serde_json::Value>, AppError> {
    let todo = state
        .todo_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    // Verify user is member of the group or admin
    let is_member = state.group_repo.is_member(todo.group_id, user.id).await?;
    if !is_member && !user.is_admin {
        return Err(AppError::Forbidden(
            "You must be a member of the group to update this todo".to_string(),
        ));
    }

    let updated_todo = state.todo_repo.update(id, &dto).await?;
    success(updated_todo)
}

pub async fn delete_todo(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let todo = state
        .todo_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    // Only creator or admin can delete
    if todo.created_by != user.id && !user.is_admin {
        return Err(AppError::Forbidden(
            "Only todo creator or admin can delete todo".to_string(),
        ));
    }

    state.todo_repo.delete(id).await?;
    success(serde_json::json!({"message": "Todo deleted successfully"}))
}

pub async fn list_group_todos(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify user is member of the group or admin
    let is_member = state.group_repo.is_member(group_id, user.id).await?;
    if !is_member && !user.is_admin {
        return Err(AppError::Forbidden(
            "You must be a member of the group to view todos".to_string(),
        ));
    }

    let todos = state.todo_repo.list_by_group(group_id).await?;
    success(todos)
}

pub async fn list_user_todos(
    AuthUser(auth_user): AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Users can only view their own todos unless they're admin
    if auth_user.id != user_id && !auth_user.is_admin {
        return Err(AppError::Forbidden(
            "You can only view your own todos".to_string(),
        ));
    }

    let todos = state.todo_repo.list_by_user(user_id).await?;
    success(todos)
}

pub async fn assign_user(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path((todo_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let todo = state
        .todo_repo
        .find_by_id(todo_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", todo_id)))?;

    // Verify requesting user is member of the group or admin
    let is_member = state.group_repo.is_member(todo.group_id, user.id).await?;
    if !is_member && !user.is_admin {
        return Err(AppError::Forbidden(
            "You must be a member of the group to assign users".to_string(),
        ));
    }

    // Verify user being assigned is member of the group
    let is_target_member = state.group_repo.is_member(todo.group_id, user_id).await?;
    if !is_target_member {
        return Err(AppError::BadRequest(
            "User must be a member of the group to be assigned to todo".to_string(),
        ));
    }

    state.todo_repo.assign_user(todo_id, user_id).await?;
    success(serde_json::json!({"message": "User assigned successfully"}))
}

pub async fn unassign_user(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path((todo_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let todo = state
        .todo_repo
        .find_by_id(todo_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", todo_id)))?;

    // Verify requesting user is member of the group or admin
    let is_member = state.group_repo.is_member(todo.group_id, user.id).await?;
    if !is_member && !user.is_admin {
        return Err(AppError::Forbidden(
            "You must be a member of the group to unassign users".to_string(),
        ));
    }

    state.todo_repo.unassign_user(todo_id, user_id).await?;
    success(serde_json::json!({"message": "User unassigned successfully"}))
}

pub async fn get_assigned_users(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let todo = state
        .todo_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    // Verify user is member of the group or admin
    let is_member = state.group_repo.is_member(todo.group_id, user.id).await?;
    if !is_member && !user.is_admin {
        return Err(AppError::Forbidden(
            "You must be a member of the group to view assigned users".to_string(),
        ));
    }

    let users = state.todo_repo.get_assigned_users(id).await?;
    success(users)
}
