use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum TodoStatus {
    Active,
    InProgress,
    Closed,
}

impl ToString for TodoStatus {
    fn to_string(&self) -> String {
        match self {
            TodoStatus::Active => "active".to_string(),
            TodoStatus::InProgress => "in_progress".to_string(),
            TodoStatus::Closed => "closed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl ToString for Priority {
    fn to_string(&self) -> String {
        match self {
            Priority::Low => "low".to_string(),
            Priority::Medium => "medium".to_string(),
            Priority::High => "high".to_string(),
            Priority::Critical => "critical".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_admin: bool,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TodoStatus,
    pub priority: Priority,
    pub notes: Option<String>,
    pub estimated_duration: Option<i32>, // in hours
    pub actual_duration: Option<i32>,    // in hours
    pub due_date: Option<DateTime<Utc>>,
    pub group_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoAssignment {
    pub todo_id: Uuid,
    pub user_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// DTOs
#[derive(Debug, Deserialize)]
pub struct CreateUserDto {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserDto {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupDto {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoDto {
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub notes: Option<String>,
    pub estimated_duration: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub group_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TodoStatus>,
    pub priority: Option<Priority>,
    pub notes: Option<String>,
    pub estimated_duration: Option<i32>,
    pub actual_duration: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub is_admin: bool,
    pub points: i32,
    pub total_todos: i64,
    pub active_todos: i64,
    pub in_progress_todos: i64,
    pub closed_todos: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GroupStats {
    pub group_id: Uuid,
    pub group_name: String,
    pub active_count: i64,
    pub in_progress_count: i64,
    pub closed_count: i64,
    pub total_estimated_duration: i64,
    pub total_actual_duration: i64,
    pub overdue_count: i64,
}
