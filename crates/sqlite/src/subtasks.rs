use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{RepositoryError, SubtaskStore};
use todooshka_domain::{
    ProjectId, Subtask, SubtaskId, SubtaskSnapshot, SubtaskTitle, TaskId, Timestamp,
};

use crate::mapping::{corrupted, unavailable};

pub struct SqliteSubtasks {
    pool: SqlitePool,
}

impl SqliteSubtasks {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SubtaskRow {
    id: i64,
    task_id: i64,
    title: String,
    done: i64,
    position: f64,
    created_at: i64,
    updated_at: i64,
}

impl SubtaskRow {
    fn into_domain(self) -> Result<Subtask, RepositoryError> {
        let snapshot = SubtaskSnapshot {
            id: SubtaskId::from_raw(self.id),
            task_id: TaskId::from_raw(self.task_id),
            title: SubtaskTitle::parse(&self.title).map_err(corrupted)?,
            done: self.done != 0,
            position: self.position,
            created_at: Timestamp::from_unix_seconds(self.created_at),
            updated_at: Timestamp::from_unix_seconds(self.updated_at),
        };

        Subtask::restore(snapshot).map_err(corrupted)
    }
}

const COLUMNS: &str = "id, task_id, title, done, position, created_at, updated_at";

#[async_trait]
impl SubtaskStore for SqliteSubtasks {
    async fn list_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<Subtask>, RepositoryError> {
        let rows: Vec<SubtaskRow> = sqlx::query_as(
            "SELECT s.id, s.task_id, s.title, s.done, s.position, s.created_at, s.updated_at
             FROM subtasks s
             JOIN tasks t ON t.id = s.task_id
             WHERE t.project_id = ?1
             ORDER BY s.task_id ASC, s.position ASC, s.id ASC",
        )
        .bind(project_id.as_raw())
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(SubtaskRow::into_domain).collect()
    }

    async fn add(
        &self,
        task_id: TaskId,
        title: &str,
        now: Timestamp,
    ) -> Result<Option<Subtask>, RepositoryError> {
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM tasks WHERE id = ?1")
            .bind(task_id.as_raw())
            .fetch_optional(&self.pool)
            .await
            .map_err(unavailable)?;
        if exists.is_none() {
            return Ok(None);
        }

        let row: SubtaskRow = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "INSERT INTO subtasks (task_id, title, done, position, created_at, updated_at)
             VALUES (
                ?1, ?2, 0,
                (SELECT COALESCE(MAX(position), 0) + 1 FROM subtasks WHERE task_id = ?1),
                ?3, ?3
             )
             RETURNING {COLUMNS}"
        )))
        .bind(task_id.as_raw())
        .bind(title)
        .bind(now.as_unix_seconds())
        .fetch_one(&self.pool)
        .await
        .map_err(unavailable)?;

        row.into_domain().map(Some)
    }

    async fn update(
        &self,
        id: SubtaskId,
        title: Option<&str>,
        done: Option<bool>,
        now: Timestamp,
    ) -> Result<Option<Subtask>, RepositoryError> {
        let row: Option<SubtaskRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {COLUMNS} FROM subtasks WHERE id = ?1"
        )))
        .bind(id.as_raw())
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let mut subtask = row.into_domain()?;
        if let Some(title) = title {
            let parsed = SubtaskTitle::parse(title).map_err(corrupted)?;
            subtask.rename(parsed, now);
        }
        if let Some(done) = done {
            subtask.set_done(done, now);
        }

        sqlx::query(
            "UPDATE subtasks
             SET title = ?1, done = ?2, updated_at = ?3
             WHERE id = ?4",
        )
        .bind(subtask.title().as_str())
        .bind(i64::from(subtask.is_done()))
        .bind(subtask.updated_at().as_unix_seconds())
        .bind(subtask.id().as_raw())
        .execute(&self.pool)
        .await
        .map_err(unavailable)?;

        Ok(Some(subtask))
    }

    async fn delete(&self, id: SubtaskId) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM subtasks WHERE id = ?1")
            .bind(id.as_raw())
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(result.rows_affected() > 0)
    }
}
