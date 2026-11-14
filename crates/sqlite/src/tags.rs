use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use todooshka_app::{RepositoryError, TagStore};
use todooshka_domain::{ProjectId, Tag, TagId, TagName, TaskId, TaskTag};

use crate::mapping::{corrupted, unavailable};

pub struct SqliteTags {
    pool: SqlitePool,
}

impl SqliteTags {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagStore for SqliteTags {
    async fn list_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<TaskTag>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            task_id: i64,
            name: String,
        }

        let rows: Vec<Row> = sqlx::query_as(
            "SELECT tt.task_id, g.name
             FROM task_tags tt
             JOIN tags g ON g.id = tt.tag_id
             JOIN tasks t ON t.id = tt.task_id
             WHERE t.project_id = ?1
             ORDER BY tt.task_id ASC, g.name ASC",
        )
        .bind(project_id.as_raw())
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskTag {
                    task_id: TaskId::from_raw(row.task_id),
                    name: TagName::parse(&row.name).map_err(corrupted)?,
                })
            })
            .collect()
    }

    async fn list_all(&self) -> Result<Vec<Tag>, RepositoryError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i64,
            name: String,
        }

        let rows: Vec<Row> = sqlx::query_as("SELECT id, name FROM tags ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(unavailable)?;

        rows.into_iter()
            .map(|row| {
                let name = TagName::parse(&row.name).map_err(corrupted)?;
                Tag::restore(TagId::from_raw(row.id), name).map_err(corrupted)
            })
            .collect()
    }

    async fn replace(
        &self,
        task_id: TaskId,
        names: &[TagName],
    ) -> Result<Option<Vec<Tag>>, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(unavailable)?;

        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM tasks WHERE id = ?1")
            .bind(task_id.as_raw())
            .fetch_optional(&mut *tx)
            .await
            .map_err(unavailable)?;
        if exists.is_none() {
            return Ok(None);
        }

        sqlx::query("DELETE FROM task_tags WHERE task_id = ?1")
            .bind(task_id.as_raw())
            .execute(&mut *tx)
            .await
            .map_err(unavailable)?;

        let mut tags = Vec::with_capacity(names.len());
        for name in names {
            tags.push(attach(&mut tx, task_id, name).await?);
        }

        tx.commit().await.map_err(unavailable)?;
        Ok(Some(tags))
    }
}

async fn attach(
    tx: &mut Transaction<'_, Sqlite>,
    task_id: TaskId,
    name: &TagName,
) -> Result<Tag, RepositoryError> {
    sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?1)")
        .bind(name.as_str())
        .execute(&mut **tx)
        .await
        .map_err(unavailable)?;

    let (id, stored): (i64, String) = sqlx::query_as("SELECT id, name FROM tags WHERE name = ?1")
        .bind(name.as_str())
        .fetch_one(&mut **tx)
        .await
        .map_err(unavailable)?;

    sqlx::query("INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)")
        .bind(task_id.as_raw())
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(unavailable)?;

    let name = TagName::parse(&stored).map_err(corrupted)?;
    Tag::restore(TagId::from_raw(id), name).map_err(corrupted)
}
