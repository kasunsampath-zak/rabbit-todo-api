use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::models::*;
use crate::domain::repositories::UserRepository;
use crate::infrastructure::error::AppError;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn create(&self, user: &User) -> Result<User, AppError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, is_admin, points, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user.id.to_string())
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(user.is_admin as i32)
        .bind(user.points)
        .bind(user.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let result = sqlx::query_as::<_, (String, String, String, i32, i32, String)>(
            r#"
            SELECT id, username, password_hash, is_admin, points, created_at
            FROM users WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(id, username, password_hash, is_admin, points, created_at)| {
            User {
                id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::nil()),
                username,
                password_hash,
                is_admin: is_admin != 0,
                points,
                created_at: created_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
            }
        }))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let result = sqlx::query_as::<_, (String, String, String, i32, i32, String)>(
            r#"
            SELECT id, username, password_hash, is_admin, points, created_at
            FROM users WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(id, username, password_hash, is_admin, points, created_at)| {
            User {
                id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::nil()),
                username,
                password_hash,
                is_admin: is_admin != 0,
                points,
                created_at: created_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
            }
        }))
    }

    async fn update(&self, id: Uuid, dto: &UpdateUserDto) -> Result<User, AppError> {
        let user = self.find_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("User with id {} not found", id))
        })?;

        let username = dto.username.as_ref().unwrap_or(&user.username);
        let password_hash = if let Some(password) = &dto.password {
            bcrypt::hash(password, bcrypt::DEFAULT_COST)
                .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?
        } else {
            user.password_hash.clone()
        };

        sqlx::query(
            r#"
            UPDATE users SET username = ?, password_hash = ?
            WHERE id = ?
            "#,
        )
        .bind(username)
        .bind(&password_hash)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or_else(|| {
            AppError::InternalError("Failed to fetch updated user".to_string())
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("User with id {} not found", id)));
        }

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<User>, AppError> {
        let results = sqlx::query_as::<_, (String, String, String, i32, i32, String)>(
            r#"
            SELECT id, username, password_hash, is_admin, points, created_at
            FROM users
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(id, username, password_hash, is_admin, points, created_at)| {
                User {
                    id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::nil()),
                    username,
                    password_hash,
                    is_admin: is_admin != 0,
                    points,
                    created_at: created_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
                }
            })
            .collect())
    }

    async fn update_points(&self, user_id: Uuid, delta: i32) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users SET points = points + ?
            WHERE id = ?
            "#,
        )
        .bind(delta)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        let user = self.find_by_id(user_id).await?.ok_or_else(|| {
            AppError::NotFound(format!("User with id {} not found", user_id))
        })?;

        let (total, active, in_progress, closed) = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active,
                SUM(CASE WHEN status = 'in_progress' THEN 1 ELSE 0 END) as in_progress,
                SUM(CASE WHEN status = 'closed' THEN 1 ELSE 0 END) as closed
            FROM todos t
            INNER JOIN todo_assignments ta ON t.id = ta.todo_id
            WHERE ta.user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or((0, 0, 0, 0));

        Ok(UserProfile {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
            points: user.points,
            total_todos: total,
            active_todos: active,
            in_progress_todos: in_progress,
            closed_todos: closed,
            created_at: user.created_at,
        })
    }
}
