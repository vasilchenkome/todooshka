use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{ProjectRepository, RepositoryError};
use todooshka_domain::{
    Project, ProjectColor, ProjectDraft, ProjectId, ProjectName, ProjectSnapshot, Timestamp,
};

use crate::mapping::{corrupted, unavailable};

/// SQLite implementation of the project port: SQL and row mapping, no business rules.
pub struct SqliteProjects {
    pool: SqlitePool,
}

impl SqliteProjects {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// A `projects` row exactly as stored; the domain object is built only through it.
#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: i64,
    name: String,
    description: String,
    color: String,
    position: i64,
    archived: i64,
    created_at: i64,
    updated_at: i64,
}

impl ProjectRow {
    fn into_domain(self) -> Result<Project, RepositoryError> {
        let name = ProjectName::parse(&self.name).map_err(corrupted)?;
        let color = ProjectColor::parse(&self.color).map_err(corrupted)?;
        let position = i32::try_from(self.position).map_err(|_| {
            RepositoryError::Corrupted(format!("позиция {} не помещается в i32", self.position))
        })?;

        let snapshot = ProjectSnapshot {
            id: ProjectId::from_raw(self.id),
            name,
            description: self.description,
            color,
            position,
            archived: self.archived != 0,
            created_at: Timestamp::from_unix_seconds(self.created_at),
            updated_at: Timestamp::from_unix_seconds(self.updated_at),
        };

        Project::restore(snapshot).map_err(corrupted)
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjects {
    async fn list(&self, include_archived: bool) -> Result<Vec<Project>, RepositoryError> {
        let rows: Vec<ProjectRow> = sqlx::query_as(
            "SELECT id, name, description, color, position, archived, created_at, updated_at
             FROM projects
             WHERE ?1 = 1 OR archived = 0
             ORDER BY position ASC, id ASC",
        )
        .bind(i64::from(include_archived))
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter().map(ProjectRow::into_domain).collect()
    }

    async fn get(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError> {
        let row: Option<ProjectRow> = sqlx::query_as(
            "SELECT id, name, description, color, position, archived, created_at, updated_at
             FROM projects
             WHERE id = ?1",
        )
        .bind(id.as_raw())
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;

        row.map(ProjectRow::into_domain).transpose()
    }

    async fn create(
        &self,
        draft: ProjectDraft,
        now: Timestamp,
    ) -> Result<Project, RepositoryError> {
        // The position is computed in the same statement as the insert: a separate SELECT would
        // leave a window where another query grabs the same slot at the end of the list.
        let row: ProjectRow = sqlx::query_as(
            "INSERT INTO projects (name, description, color, position, archived, created_at, updated_at)
             VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(position), -1) + 1 FROM projects), 0, ?4, ?5)
             RETURNING id, name, description, color, position, archived, created_at, updated_at",
        )
        .bind(draft.name().as_str())
        .bind(draft.description())
        .bind(draft.color().as_str())
        .bind(now.as_unix_seconds())
        .bind(now.as_unix_seconds())
        .fetch_one(&self.pool)
        .await
        .map_err(unavailable)?;

        row.into_domain()
    }

    async fn save(&self, project: &Project) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            "UPDATE projects
             SET name = ?1, description = ?2, color = ?3, position = ?4, archived = ?5,
                 updated_at = ?6
             WHERE id = ?7",
        )
        .bind(project.name().as_str())
        .bind(project.description())
        .bind(project.color().as_str())
        .bind(project.position())
        .bind(i64::from(project.is_archived()))
        .bind(project.updated_at().as_unix_seconds())
        .bind(project.id().as_raw())
        .execute(&self.pool)
        .await
        .map_err(unavailable)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict(format!(
                "проект {} не найден при сохранении",
                project.id()
            )));
        }

        Ok(())
    }
}
