use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::models::*;
use crate::domain::repositories::TodoRepository;
use crate::infrastructure::error::AppError;

pub struct SqliteTodoRepository {
    pool: SqlitePool,
}

impl SqliteTodoRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TodoRepository for SqliteTodoRepository {
    async fn create(&self, todo: &Todo) -> Result<Todo, AppError> {
        sqlx::query(
            r#"
            INSERT INTO todos (id, title, description, status, priority, notes, 
                              estimated_duration, actual_duration, due_date, group_id, 
                              created_by, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(todo.id.to_string())
        .bind(&todo.title)
        .bind(&todo.description)
        .bind(todo.status.to_string())
        .bind(todo.priority.to_string())
        .bind(&todo.notes)
        .bind(todo.estimated_duration)
        .bind(todo.actual_duration)
        .bind(todo.due_date.map(|d| d.to_rfc3339()))
        .bind(todo.group_id.to_string())
        .bind(todo.created_by.to_string())
        .bind(todo.created_at.to_rfc3339())
        .bind(todo.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Record initial status
        sqlx::query(
            r#"
            INSERT INTO todo_status_history (todo_id, previous_status, new_status, changed_at)
            VALUES (?, NULL, ?, ?)
            "#,
        )
        .bind(todo.id.to_string())
        .bind(todo.status.to_string())
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(todo.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Todo>, AppError> {
        let result = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<i32>,
            Option<i32>,
            Option<String>,
            String,
            String,
            String,
            String,
        )>(
            r#"
            SELECT id, title, description, status, priority, notes, 
                   estimated_duration, actual_duration, due_date, group_id, 
                   created_by, created_at, updated_at
            FROM todos WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(
            |(
                id,
                title,
                description,
                status,
                priority,
                notes,
                estimated_duration,
                actual_duration,
                due_date,
                group_id,
                created_by,
                created_at,
                updated_at,
            )| {
                Todo {
                    id: Uuid::parse_str(&id).unwrap(),
                    title,
                    description,
                    status: match status.as_str() {
                        "active" => TodoStatus::Active,
                        "in_progress" => TodoStatus::InProgress,
                        "closed" => TodoStatus::Closed,
                        _ => TodoStatus::Active,
                    },
                    priority: match priority.as_str() {
                        "low" => Priority::Low,
                        "medium" => Priority::Medium,
                        "high" => Priority::High,
                        "critical" => Priority::Critical,
                        _ => Priority::Medium,
                    },
                    notes,
                    estimated_duration,
                    actual_duration,
                    due_date: due_date.and_then(|d| d.parse().ok()),
                    group_id: Uuid::parse_str(&group_id).unwrap(),
                    created_by: Uuid::parse_str(&created_by).unwrap(),
                    created_at: created_at.parse().unwrap(),
                    updated_at: updated_at.parse().unwrap(),
                }
            },
        ))
    }

    async fn update(&self, id: Uuid, dto: &UpdateTodoDto) -> Result<Todo, AppError> {
        let todo = self.find_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("Todo with id {} not found", id))
        })?;

        let title = dto.title.as_ref().unwrap_or(&todo.title);
        let description = dto.description.as_ref().or(todo.description.as_ref());
        let status = dto.status.as_ref().unwrap_or(&todo.status);
        let priority = dto.priority.as_ref().unwrap_or(&todo.priority);
        let notes = dto.notes.as_ref().or(todo.notes.as_ref());
        let estimated_duration = dto.estimated_duration.or(todo.estimated_duration);
        let actual_duration = dto.actual_duration.or(todo.actual_duration);
        let due_date = dto.due_date.or(todo.due_date);

        // Check if status changed
        if status != &todo.status {
            // Record status change
            sqlx::query(
                r#"
                INSERT INTO todo_status_history (todo_id, previous_status, new_status, changed_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(id.to_string())
            .bind(todo.status.to_string())
            .bind(status.to_string())
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;

            // Handle points: +2 when closing, -2 when reopening from closed
            if *status == TodoStatus::Closed && todo.status != TodoStatus::Closed {
                // Closing: add 2 points to all assigned users
                let assigned_users = self.get_assigned_users(id).await?;
                for user in assigned_users {
                    sqlx::query("UPDATE users SET points = points + 2 WHERE id = ?")
                        .bind(user.id.to_string())
                        .execute(&self.pool)
                        .await?;
                }
            } else if todo.status == TodoStatus::Closed && *status != TodoStatus::Closed {
                // Reopening: subtract 2 points from all assigned users
                let assigned_users = self.get_assigned_users(id).await?;
                for user in assigned_users {
                    sqlx::query("UPDATE users SET points = points - 2 WHERE id = ?")
                        .bind(user.id.to_string())
                        .execute(&self.pool)
                        .await?;
                }
            }
        }

        sqlx::query(
            r#"
            UPDATE todos SET title = ?, description = ?, status = ?, priority = ?, 
                            notes = ?, estimated_duration = ?, actual_duration = ?, 
                            due_date = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(title)
        .bind(description)
        .bind(status.to_string())
        .bind(priority.to_string())
        .bind(notes)
        .bind(estimated_duration)
        .bind(actual_duration)
        .bind(due_date.map(|d| d.to_rfc3339()))
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or_else(|| {
            AppError::InternalError("Failed to fetch updated todo".to_string())
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Todo with id {} not found", id)));
        }

        Ok(())
    }

    async fn list_by_group(&self, group_id: Uuid) -> Result<Vec<Todo>, AppError> {
        let results = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<i32>,
            Option<i32>,
            Option<String>,
            String,
            String,
            String,
            String,
        )>(
            r#"
            SELECT id, title, description, status, priority, notes, 
                   estimated_duration, actual_duration, due_date, group_id, 
                   created_by, created_at, updated_at
            FROM todos WHERE group_id = ?
            "#,
        )
        .bind(group_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(self.map_todos(results))
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Todo>, AppError> {
        let results = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<i32>,
            Option<i32>,
            Option<String>,
            String,
            String,
            String,
            String,
        )>(
            r#"
            SELECT t.id, t.title, t.description, t.status, t.priority, t.notes, 
                   t.estimated_duration, t.actual_duration, t.due_date, t.group_id, 
                   t.created_by, t.created_at, t.updated_at
            FROM todos t
            INNER JOIN todo_assignments ta ON t.id = ta.todo_id
            WHERE ta.user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(self.map_todos(results))
    }

    async fn assign_user(&self, todo_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO todo_assignments (todo_id, user_id, assigned_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(todo_id.to_string())
        .bind(user_id.to_string())
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn unassign_user(&self, todo_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM todo_assignments WHERE todo_id = ? AND user_id = ?
            "#,
        )
        .bind(todo_id.to_string())
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "User is not assigned to this todo".to_string(),
            ));
        }

        Ok(())
    }

    async fn get_assigned_users(&self, todo_id: Uuid) -> Result<Vec<User>, AppError> {
        let results = sqlx::query_as::<_, (String, String, String, i32, i32, String)>(
            r#"
            SELECT u.id, u.username, u.password_hash, u.is_admin, u.points, u.created_at
            FROM users u
            INNER JOIN todo_assignments ta ON u.id = ta.user_id
            WHERE ta.todo_id = ?
            "#,
        )
        .bind(todo_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(id, username, password_hash, is_admin, points, created_at)| User {
                id: Uuid::parse_str(&id).unwrap(),
                username,
                password_hash,
                is_admin: is_admin != 0,
                points,
                created_at: created_at.parse().unwrap(),
            })
            .collect())
    }

    async fn get_group_stats(&self, group_id: Uuid) -> Result<GroupStats, AppError> {
        let group = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM groups WHERE id = ?",
        )
        .bind(group_id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Group with id {} not found", group_id)))?;

        let stats = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            r#"
            SELECT 
                SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active_count,
                SUM(CASE WHEN status = 'in_progress' THEN 1 ELSE 0 END) as in_progress_count,
                SUM(CASE WHEN status = 'closed' THEN 1 ELSE 0 END) as closed_count,
                COALESCE(SUM(estimated_duration), 0) as total_estimated_duration,
                COALESCE(SUM(actual_duration), 0) as total_actual_duration
            FROM todos WHERE group_id = ?
            "#,
        )
        .bind(group_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        let overdue = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM todos 
            WHERE group_id = ? AND due_date < ? AND status != 'closed'
            "#,
        )
        .bind(group_id.to_string())
        .bind(Utc::now().to_rfc3339())
        .fetch_one(&self.pool)
        .await?;

        Ok(GroupStats {
            group_id,
            group_name: group.1,
            active_count: stats.0,
            in_progress_count: stats.1,
            closed_count: stats.2,
            total_estimated_duration: stats.3,
            total_actual_duration: stats.4,
            overdue_count: overdue.0,
        })
    }

    async fn get_previous_status(&self, todo_id: Uuid) -> Result<Option<TodoStatus>, AppError> {
        let result = sqlx::query_as::<_, (Option<String>,)>(
            r#"
            SELECT previous_status FROM todo_status_history
            WHERE todo_id = ?
            ORDER BY changed_at DESC
            LIMIT 1
            "#,
        )
        .bind(todo_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.and_then(|(status,)| {
            status.map(|s| match s.as_str() {
                "active" => TodoStatus::Active,
                "in_progress" => TodoStatus::InProgress,
                "closed" => TodoStatus::Closed,
                _ => TodoStatus::Active,
            })
        }))
    }
}

impl SqliteTodoRepository {
    fn map_todos(
        &self,
        results: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<i32>,
            Option<i32>,
            Option<String>,
            String,
            String,
            String,
            String,
        )>,
    ) -> Vec<Todo> {
        results
            .into_iter()
            .map(
                |(
                    id,
                    title,
                    description,
                    status,
                    priority,
                    notes,
                    estimated_duration,
                    actual_duration,
                    due_date,
                    group_id,
                    created_by,
                    created_at,
                    updated_at,
                )| {
                    Todo {
                        id: Uuid::parse_str(&id).unwrap(),
                        title,
                        description,
                        status: match status.as_str() {
                            "active" => TodoStatus::Active,
                            "in_progress" => TodoStatus::InProgress,
                            "closed" => TodoStatus::Closed,
                            _ => TodoStatus::Active,
                        },
                        priority: match priority.as_str() {
                            "low" => Priority::Low,
                            "medium" => Priority::Medium,
                            "high" => Priority::High,
                            "critical" => Priority::Critical,
                            _ => Priority::Medium,
                        },
                        notes,
                        estimated_duration,
                        actual_duration,
                        due_date: due_date.and_then(|d| d.parse().ok()),
                        group_id: Uuid::parse_str(&group_id).unwrap(),
                        created_by: Uuid::parse_str(&created_by).unwrap(),
                        created_at: created_at.parse().unwrap(),
                        updated_at: updated_at.parse().unwrap(),
                    }
                },
            )
            .collect()
    }
}
