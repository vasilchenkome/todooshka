use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use todooshka_app::{ProjectTaskCount, RepositoryError, TaskStore, TaskTransaction};
use todooshka_domain::{
    DueDate, EstimateMinutes, Planning, Priority, ProjectId, Task, TaskDraft, TaskId, TaskSnapshot,
    TaskStatus, TaskTitle, Timestamp,
};

use crate::mapping::{corrupted, unavailable};

/// SQLite implementation of the task port. A move runs as a Unit of Work:
/// the transaction opens here and the use case commits it.
pub struct SqliteTasks {
    pool: SqlitePool,
}

impl SqliteTasks {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct TaskRow {
    pub(crate) id: i64,
    pub(crate) project_id: i64,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) status: String,
    pub(crate) position: f64,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
    pub(crate) completed_at: Option<i64>,
    pub(crate) due_on: Option<String>,
    pub(crate) priority: String,
    pub(crate) estimate_minutes: Option<i64>,
}

impl TaskRow {
    pub(crate) fn into_domain(self) -> Result<Task, RepositoryError> {
        let status = TaskStatus::from_code(&self.status).ok_or_else(|| {
            RepositoryError::Corrupted(format!("неизвестный статус задачи {:?}", self.status))
        })?;
        let priority = Priority::from_code(&self.priority).ok_or_else(|| {
            RepositoryError::Corrupted(format!("неизвестный приоритет задачи {:?}", self.priority))
        })?;
        let due_on = self
            .due_on
            .as_deref()
            .map(DueDate::parse)
            .transpose()
            .map_err(corrupted)?;
        let estimate = self
            .estimate_minutes
            .map(EstimateMinutes::parse)
            .transpose()
            .map_err(corrupted)?;

        let snapshot = TaskSnapshot {
            id: TaskId::from_raw(self.id),
            project_id: ProjectId::from_raw(self.project_id),
            title: TaskTitle::parse(&self.title).map_err(corrupted)?,
            description: self.description,
            status,
            position: self.position,
            created_at: Timestamp::from_unix_seconds(self.created_at),
            updated_at: Timestamp::from_unix_seconds(self.updated_at),
            completed_at: self.completed_at.map(Timestamp::from_unix_seconds),
            planning: Planning::new(due_on, priority, estimate),
        };

        Task::restore(snapshot).map_err(corrupted)
    }
}

#[async_trait]
impl TaskStore for SqliteTasks {
    async fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<Task>, RepositoryError> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            "SELECT id, project_id, title, description, status, position,
                    created_at, updated_at, completed_at, due_on, priority, estimate_minutes
             FROM tasks
             WHERE project_id = ?1
             ORDER BY position ASC, id ASC",
        )
        .bind(project_id.as_raw())
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(TaskRow::into_domain).collect()
    }

    async fn delete(&self, id: TaskId) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?1")
            .bind(id.as_raw())
            .execute(&self.pool)
            .await
            .map_err(unavailable)?;

        Ok(result.rows_affected() > 0)
    }

    async fn counts_by_project(&self) -> Result<Vec<ProjectTaskCount>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct CountRow {
            project_id: i64,
            status: String,
            tasks: i64,
        }

        let rows: Vec<CountRow> = sqlx::query_as(
            "SELECT project_id, status, COUNT(*) AS tasks
             FROM tasks
             GROUP BY project_id, status",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter()
            .map(|row| {
                let status = TaskStatus::from_code(&row.status).ok_or_else(|| {
                    RepositoryError::Corrupted(format!(
                        "неизвестный статус задачи {:?}",
                        row.status
                    ))
                })?;
                Ok(ProjectTaskCount {
                    project_id: ProjectId::from_raw(row.project_id),
                    status,
                    tasks: row.tasks,
                })
            })
            .collect()
    }

    async fn transaction(&self) -> Result<Box<dyn TaskTransaction>, RepositoryError> {
        let tx = self.pool.begin().await.map_err(unavailable)?;
        Ok(Box::new(SqliteTaskTransaction { tx: Some(tx) }))
    }
}

struct SqliteTaskTransaction {
    tx: Option<Transaction<'static, Sqlite>>,
}

impl SqliteTaskTransaction {
    fn tx(&mut self) -> Result<&mut Transaction<'static, Sqlite>, RepositoryError> {
        self.tx
            .as_mut()
            .ok_or_else(|| RepositoryError::Conflict("транзакция уже завершена".to_owned()))
    }
}

#[async_trait]
impl TaskTransaction for SqliteTaskTransaction {
    async fn list_column(
        &mut self,
        project_id: ProjectId,
        status: TaskStatus,
    ) -> Result<Vec<Task>, RepositoryError> {
        let rows: Vec<TaskRow> = sqlx::query_as(
            "SELECT id, project_id, title, description, status, position,
                    created_at, updated_at, completed_at, due_on, priority, estimate_minutes
             FROM tasks
             WHERE project_id = ?1 AND status = ?2
             ORDER BY position ASC, id ASC",
        )
        .bind(project_id.as_raw())
        .bind(status.as_code())
        .fetch_all(&mut **self.tx()?)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(TaskRow::into_domain).collect()
    }

    async fn get(&mut self, id: TaskId) -> Result<Option<Task>, RepositoryError> {
        let row: Option<TaskRow> = sqlx::query_as(
            "SELECT id, project_id, title, description, status, position,
                    created_at, updated_at, completed_at, due_on, priority, estimate_minutes
             FROM tasks
             WHERE id = ?1",
        )
        .bind(id.as_raw())
        .fetch_optional(&mut **self.tx()?)
        .await
        .map_err(unavailable)?;

        row.map(TaskRow::into_domain).transpose()
    }

    async fn create(
        &mut self,
        project_id: ProjectId,
        draft: TaskDraft,
        status: TaskStatus,
        now: Timestamp,
    ) -> Result<Task, RepositoryError> {
        let completed_at = status.is_done().then(|| now.as_unix_seconds());
        let tx = self.tx()?;

        // The end-of-column position is computed by the same query as the insert.
        let row: TaskRow = sqlx::query_as(
            "INSERT INTO tasks
                (project_id, title, description, status, position, created_at, updated_at,
                 completed_at)
             VALUES (
                ?1, ?2, ?3, ?4,
                (SELECT COALESCE(MAX(position), 0) + 1
                 FROM tasks WHERE project_id = ?1 AND status = ?4),
                ?5, ?5, ?6
             )
             RETURNING id, project_id, title, description, status, position,
                       created_at, updated_at, completed_at, due_on, priority, estimate_minutes",
        )
        .bind(project_id.as_raw())
        .bind(draft.title().as_str())
        .bind(draft.description())
        .bind(status.as_code())
        .bind(now.as_unix_seconds())
        .bind(completed_at)
        .fetch_one(&mut **tx)
        .await
        .map_err(unavailable)?;

        row.into_domain()
    }

    async fn save(&mut self, task: &Task) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "UPDATE tasks
             SET title = ?1, description = ?2, status = ?3, position = ?4,
                 updated_at = ?5, completed_at = ?6, due_on = ?7, priority = ?8,
                 estimate_minutes = ?9
             WHERE id = ?10",
        )
        .bind(task.title().as_str())
        .bind(task.description())
        .bind(task.status().as_code())
        .bind(task.position())
        .bind(task.updated_at().as_unix_seconds())
        .bind(task.completed_at().map(Timestamp::as_unix_seconds))
        .bind(task.planning().due_on().map(DueDate::as_str))
        .bind(task.planning().priority().as_code())
        .bind(task.planning().estimate().map(EstimateMinutes::as_minutes))
        .bind(task.id().as_raw())
        .execute(&mut **self.tx()?)
        .await
        .map_err(unavailable)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict(format!(
                "задача {} не найдена при сохранении",
                task.id()
            )));
        }

        Ok(())
    }

    async fn commit(&mut self) -> Result<(), RepositoryError> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| RepositoryError::Conflict("транзакция уже завершена".to_owned()))?;
        tx.commit().await.map_err(unavailable)
    }
}
