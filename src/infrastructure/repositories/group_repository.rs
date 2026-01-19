use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

use crate::domain::models::*;
use crate::domain::repositories::GroupRepository;
use crate::infrastructure::error::AppError;

pub struct SqliteGroupRepository {
    pool: SqlitePool,
}

impl SqliteGroupRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepository for SqliteGroupRepository {
    async fn create(&self, group: &Group) -> Result<Group, AppError> {
        sqlx::query(
            r#"
            INSERT INTO groups (id, name, description, created_by, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(group.id.to_string())
        .bind(&group.name)
        .bind(&group.description)
        .bind(group.created_by.to_string())
        .bind(group.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(group.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Group>, AppError> {
        let result = sqlx::query_as::<_, (String, String, Option<String>, String, String)>(
            r#"
            SELECT id, name, description, created_by, created_at
            FROM groups WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(id, name, description, created_by, created_at)| Group {
            id: Uuid::parse_str(&id).unwrap(),
            name,
            description,
            created_by: Uuid::parse_str(&created_by).unwrap(),
            created_at: created_at.parse().unwrap(),
        }))
    }

    async fn update(&self, id: Uuid, dto: &UpdateGroupDto) -> Result<Group, AppError> {
        let group = self.find_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("Group with id {} not found", id))
        })?;

        let name = dto.name.as_ref().unwrap_or(&group.name);
        let description = dto.description.as_ref().or(group.description.as_ref());

        sqlx::query(
            r#"
            UPDATE groups SET name = ?, description = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or_else(|| {
            AppError::InternalError("Failed to fetch updated group".to_string())
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM groups WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Group with id {} not found", id)));
        }

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Group>, AppError> {
        let results = sqlx::query_as::<_, (String, String, Option<String>, String, String)>(
            r#"
            SELECT id, name, description, created_by, created_at
            FROM groups
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(id, name, description, created_by, created_at)| Group {
                id: Uuid::parse_str(&id).unwrap(),
                name,
                description,
                created_by: Uuid::parse_str(&created_by).unwrap(),
                created_at: created_at.parse().unwrap(),
            })
            .collect())
    }

    async fn add_member(&self, group_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO group_members (group_id, user_id, joined_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(group_id.to_string())
        .bind(user_id.to_string())
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_member(&self, group_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM group_members WHERE group_id = ? AND user_id = ?
            "#,
        )
        .bind(group_id.to_string())
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "User is not a member of this group".to_string(),
            ));
        }

        Ok(())
    }

    async fn get_members(&self, group_id: Uuid) -> Result<Vec<User>, AppError> {
        let results = sqlx::query_as::<_, (String, String, String, i32, i32, String)>(
            r#"
            SELECT u.id, u.username, u.password_hash, u.is_admin, u.points, u.created_at
            FROM users u
            INNER JOIN group_members gm ON u.id = gm.user_id
            WHERE gm.group_id = ?
            "#,
        )
        .bind(group_id.to_string())
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

    async fn is_member(&self, group_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM group_members
            WHERE group_id = ? AND user_id = ?
            "#,
        )
        .bind(group_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0 > 0)
    }
}
