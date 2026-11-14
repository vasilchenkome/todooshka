use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{AttachmentStore, RepositoryError};
use todooshka_domain::{
    AttachmentFilename, AttachmentId, AttachmentMeta, ProjectId, TaskId, Timestamp,
};

use crate::mapping::{corrupted, unavailable};

pub struct SqliteAttachments {
    pool: SqlitePool,
}

impl SqliteAttachments {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AttachmentRow {
    id: i64,
    task_id: i64,
    filename: String,
    content_type: String,
    size: i64,
    created_at: i64,
}

impl AttachmentRow {
    fn into_meta(self) -> Result<AttachmentMeta, RepositoryError> {
        Ok(AttachmentMeta {
            id: AttachmentId::from_raw(self.id),
            task_id: TaskId::from_raw(self.task_id),
            filename: AttachmentFilename::parse(&self.filename).map_err(corrupted)?,
            content_type: self.content_type,
            size: self.size,
            created_at: Timestamp::from_unix_seconds(self.created_at),
        })
    }
}

const META_COLUMNS: &str = "id, task_id, filename, content_type, size, created_at";

#[async_trait]
impl AttachmentStore for SqliteAttachments {
    async fn list_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<AttachmentMeta>, RepositoryError> {
        let rows: Vec<AttachmentRow> = sqlx::query_as(
            "SELECT a.id, a.task_id, a.filename, a.content_type, a.size, a.created_at
             FROM attachments a
             JOIN tasks t ON t.id = a.task_id
             WHERE t.project_id = ?1
             ORDER BY a.task_id ASC, a.id ASC",
        )
        .bind(project_id.as_raw())
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(AttachmentRow::into_meta).collect()
    }

    async fn count_for_task(&self, task_id: TaskId) -> Result<i64, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attachments WHERE task_id = ?1")
            .bind(task_id.as_raw())
            .fetch_one(&self.pool)
            .await
            .map_err(unavailable)?;
        Ok(count)
    }

    async fn add(
        &self,
        task_id: TaskId,
        filename: &str,
        content_type: &str,
        data: &[u8],
        now: Timestamp,
    ) -> Result<Option<AttachmentMeta>, RepositoryError> {
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM tasks WHERE id = ?1")
            .bind(task_id.as_raw())
            .fetch_optional(&self.pool)
            .await
            .map_err(unavailable)?;
        if exists.is_none() {
            return Ok(None);
        }

        let row: AttachmentRow = sqlx::query_as(&format!(
            "INSERT INTO attachments (task_id, filename, content_type, size, data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             RETURNING {META_COLUMNS}"
        ))
        .bind(task_id.as_raw())
        .bind(filename)
        .bind(content_type)
        .bind(data.len() as i64)
        .bind(data)
        .bind(now.as_unix_seconds())
        .fetch_one(&self.pool)
        .await
        .map_err(unavailable)?;

        row.into_meta().map(Some)
    }

    async fn get(
        &self,
        id: AttachmentId,
    ) -> Result<Option<(AttachmentMeta, Vec<u8>)>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i64,
            task_id: i64,
            filename: String,
            content_type: String,
            size: i64,
            created_at: i64,
            data: Vec<u8>,
        }

        let row: Option<Row> = sqlx::query_as(
            "SELECT id, task_id, filename, content_type, size, created_at, data
             FROM attachments WHERE id = ?1",
        )
        .bind(id.as_raw())
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let data = row.data.clone();
        let meta = AttachmentRow {
            id: row.id,
            task_id: row.task_id,
            filename: row.filename,
            content_type: row.content_type,
            size: row.size,
            created_at: row.created_at,
        }
        .into_meta()?;

        Ok(Some((meta, data)))
    }

    async fn delete(&self, id: AttachmentId) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM attachments WHERE id = ?1")
            .bind(id.as_raw())
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(result.rows_affected() > 0)
    }
}
