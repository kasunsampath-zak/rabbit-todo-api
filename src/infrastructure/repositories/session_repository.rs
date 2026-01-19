use async_trait::async_trait;
use sqlx::SqlitePool;
use chrono::Utc;

use crate::domain::models::Session;
use crate::domain::repositories::SessionRepository;
use crate::infrastructure::error::AppError;

pub struct SqliteSessionRepository {
    pool: SqlitePool,
}

impl SqliteSessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn create(&self, session: &Session) -> Result<Session, AppError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, created_at, expires_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(session.user_id.to_string())
        .bind(session.created_at.to_rfc3339())
        .bind(session.expires_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(session.clone())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, AppError> {
        let result = sqlx::query_as::<_, (String, String, String, String)>(
            r#"
            SELECT id, user_id, created_at, expires_at
            FROM sessions WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(id, user_id, created_at, expires_at)| Session {
            id,
            user_id: user_id.parse().unwrap(),
            created_at: created_at.parse().unwrap(),
            expires_at: expires_at.parse().unwrap(),
        }))
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
