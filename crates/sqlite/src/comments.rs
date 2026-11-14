use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{CommentStore, RepositoryError};
use todooshka_domain::{
    Comment, CommentBody, CommentId, CommentSnapshot, ProjectId, TaskId, Timestamp,
};

use crate::mapping::{corrupted, unavailable};

pub struct SqliteComments {
    pool: SqlitePool,
}

impl SqliteComments {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CommentRow {
    id: i64,
    task_id: i64,
    body: String,
    created_at: i64,
    updated_at: i64,
}

impl CommentRow {
    fn into_domain(self) -> Result<Comment, RepositoryError> {
        let snapshot = CommentSnapshot {
            id: CommentId::from_raw(self.id),
            task_id: TaskId::from_raw(self.task_id),
            body: CommentBody::parse(&self.body).map_err(corrupted)?,
            created_at: Timestamp::from_unix_seconds(self.created_at),
            updated_at: Timestamp::from_unix_seconds(self.updated_at),
        };

        Ok(Comment::restore(snapshot))
    }
}

const COLUMNS: &str = "id, task_id, body, created_at, updated_at";

#[async_trait]
impl CommentStore for SqliteComments {
    async fn list_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<Comment>, RepositoryError> {
        let rows: Vec<CommentRow> = sqlx::query_as(
            "SELECT c.id, c.task_id, c.body, c.created_at, c.updated_at
             FROM comments c
             JOIN tasks t ON t.id = c.task_id
             WHERE t.project_id = ?1
             ORDER BY c.created_at ASC, c.id ASC",
        )
        .bind(project_id.as_raw())
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(CommentRow::into_domain).collect()
    }

    async fn add(
        &self,
        task_id: TaskId,
        body: &str,
        now: Timestamp,
    ) -> Result<Option<Comment>, RepositoryError> {
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM tasks WHERE id = ?1")
            .bind(task_id.as_raw())
            .fetch_optional(&self.pool)
            .await
            .map_err(unavailable)?;
        if exists.is_none() {
            return Ok(None);
        }

        let row: CommentRow = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "INSERT INTO comments (task_id, body, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)
             RETURNING {COLUMNS}"
        )))
        .bind(task_id.as_raw())
        .bind(body)
        .bind(now.as_unix_seconds())
        .fetch_one(&self.pool)
        .await
        .map_err(unavailable)?;

        row.into_domain().map(Some)
    }

    async fn edit(
        &self,
        id: CommentId,
        body: &str,
        now: Timestamp,
    ) -> Result<Option<Comment>, RepositoryError> {
        let row: Option<CommentRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {COLUMNS} FROM comments WHERE id = ?1"
        )))
        .bind(id.as_raw())
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let mut comment = row.into_domain()?;
        comment.edit(CommentBody::parse(body).map_err(corrupted)?, now);

        sqlx::query("UPDATE comments SET body = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(comment.body().as_str())
            .bind(comment.updated_at().as_unix_seconds())
            .bind(comment.id().as_raw())
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(Some(comment))
    }

    async fn delete(&self, id: CommentId) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM comments WHERE id = ?1")
            .bind(id.as_raw())
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(result.rows_affected() > 0)
    }
}
