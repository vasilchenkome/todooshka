use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{AgendaEntry, AgendaReader, RepositoryError};
use todooshka_domain::{DueDate, ProjectId};

use crate::mapping::unavailable;
use crate::tasks::TaskRow;

/// Read-only query behind the Today screen: one join, no transactions.
pub struct SqliteAgenda {
    pool: SqlitePool,
}

impl SqliteAgenda {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AgendaRow {
    id: i64,
    project_id: i64,
    project_name: String,
    title: String,
    description: String,
    status: String,
    position: f64,
    created_at: i64,
    updated_at: i64,
    completed_at: Option<i64>,
    due_on: Option<String>,
    priority: String,
    estimate_minutes: Option<i64>,
}

impl AgendaRow {
    fn into_entry(self) -> Result<AgendaEntry, RepositoryError> {
        let project_id = ProjectId::from_raw(self.project_id);
        let task = TaskRow {
            id: self.id,
            project_id: self.project_id,
            title: self.title,
            description: self.description,
            status: self.status,
            position: self.position,
            created_at: self.created_at,
            updated_at: self.updated_at,
            completed_at: self.completed_at,
            due_on: self.due_on,
            priority: self.priority,
            estimate_minutes: self.estimate_minutes,
        }
        .into_domain()?;

        Ok(AgendaEntry {
            project_id,
            project_name: self.project_name,
            task,
        })
    }
}

#[async_trait]
impl AgendaReader for SqliteAgenda {
    async fn list_open_due_until(
        &self,
        through: &DueDate,
    ) -> Result<Vec<AgendaEntry>, RepositoryError> {
        let rows: Vec<AgendaRow> = sqlx::query_as(
            "SELECT t.id, t.project_id, p.name AS project_name, t.title, t.description,
                    t.status, t.position, t.created_at, t.updated_at, t.completed_at,
                    t.due_on, t.priority, t.estimate_minutes
             FROM tasks t
             JOIN projects p ON p.id = t.project_id
             WHERE t.status <> 'done'
               AND t.due_on IS NOT NULL
               AND t.due_on <= ?1",
        )
        .bind(through.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(AgendaRow::into_entry).collect()
    }
}
