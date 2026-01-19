use async_trait::async_trait;
use uuid::Uuid;

use super::models::*;
use crate::infrastructure::error::AppError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<User, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn update(&self, id: Uuid, dto: &UpdateUserDto) -> Result<User, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn list_all(&self) -> Result<Vec<User>, AppError>;
    async fn update_points(&self, user_id: Uuid, delta: i32) -> Result<(), AppError>;
    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError>;
}

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn create(&self, group: &Group) -> Result<Group, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Group>, AppError>;
    async fn update(&self, id: Uuid, dto: &UpdateGroupDto) -> Result<Group, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn list_all(&self) -> Result<Vec<Group>, AppError>;
    async fn add_member(&self, group_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn remove_member(&self, group_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn get_members(&self, group_id: Uuid) -> Result<Vec<User>, AppError>;
    async fn is_member(&self, group_id: Uuid, user_id: Uuid) -> Result<bool, AppError>;
}

#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn create(&self, todo: &Todo) -> Result<Todo, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Todo>, AppError>;
    async fn update(&self, id: Uuid, dto: &UpdateTodoDto) -> Result<Todo, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn list_by_group(&self, group_id: Uuid) -> Result<Vec<Todo>, AppError>;
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Todo>, AppError>;
    async fn assign_user(&self, todo_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn unassign_user(&self, todo_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn get_assigned_users(&self, todo_id: Uuid) -> Result<Vec<User>, AppError>;
    async fn get_group_stats(&self, group_id: Uuid) -> Result<GroupStats, AppError>;
    async fn get_previous_status(&self, todo_id: Uuid) -> Result<Option<TodoStatus>, AppError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> Result<Session, AppError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
    async fn delete_expired(&self) -> Result<(), AppError>;
}
