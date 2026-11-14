use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::SqlitePool;
use todooshka_app::{
    ExportAttachment, ExportComment, ExportData, ExportProject, ExportReader, ExportSubtask,
    ExportTag, ExportTask, RepositoryError,
};

use crate::mapping::unavailable;

/// Builds the one plain export tree: projects → tasks → nested rows.
pub struct SqliteExport {
    pool: SqlitePool,
}

impl SqliteExport {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: i64,
    name: String,
    description: String,
    color: String,
    position: i32,
    archived: bool,
    created_at: i64,
    updated_at: i64,
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: i64,
    project_id: i64,
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

#[derive(sqlx::FromRow)]
struct SubtaskRow {
    id: i64,
    task_id: i64,
    title: String,
    done: bool,
    position: f64,
    created_at: i64,
    updated_at: i64,
}

#[derive(sqlx::FromRow)]
struct TagRow {
    id: i64,
    name: String,
}

#[derive(sqlx::FromRow)]
struct TaskTagRow {
    task_id: i64,
    name: String,
}

#[derive(sqlx::FromRow)]
struct CommentRow {
    id: i64,
    task_id: i64,
    body: String,
    created_at: i64,
    updated_at: i64,
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

#[async_trait]
impl ExportReader for SqliteExport {
    async fn read(&self) -> Result<ExportData, RepositoryError> {
        let project_rows: Vec<ProjectRow> = sqlx::query_as(
            "SELECT id, name, description, color, position, archived, created_at, updated_at
             FROM projects
             ORDER BY position ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        let mut projects = project_rows
            .into_iter()
            .map(|row| ExportProject {
                id: row.id,
                name: row.name,
                description: row.description,
                color: row.color,
                position: row.position,
                archived: row.archived,
                created_at: row.created_at,
                updated_at: row.updated_at,
                tasks: Vec::new(),
            })
            .collect::<Vec<_>>();
        let project_index = projects
            .iter()
            .enumerate()
            .map(|(index, project)| (project.id, index))
            .collect::<HashMap<_, _>>();

        let task_rows: Vec<TaskRow> = sqlx::query_as(
            "SELECT id, project_id, title, description, status, position,
                    created_at, updated_at, completed_at, due_on, priority, estimate_minutes
             FROM tasks
             ORDER BY project_id,
                      CASE status WHEN 'backlog' THEN 0 WHEN 'in_progress' THEN 1 ELSE 2 END,
                      position ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;

        let mut task_index = HashMap::new();
        for row in task_rows {
            let Some(project_position) = project_index.get(&row.project_id).copied() else {
                continue;
            };
            let task = ExportTask {
                id: row.id,
                project_id: row.project_id,
                title: row.title,
                description: row.description,
                status: row.status,
                position: row.position,
                created_at: row.created_at,
                updated_at: row.updated_at,
                completed_at: row.completed_at,
                due_on: row.due_on,
                priority: row.priority,
                estimate_minutes: row.estimate_minutes,
                subtasks: Vec::new(),
                tags: Vec::new(),
                comments: Vec::new(),
                attachments: Vec::new(),
            };
            let task_position = projects[project_position].tasks.len();
            projects[project_position].tasks.push(task);
            task_index.insert(row.id, (project_position, task_position));
        }

        let subtask_rows: Vec<SubtaskRow> = sqlx::query_as(
            "SELECT id, task_id, title, done, position, created_at, updated_at
             FROM subtasks
             ORDER BY task_id ASC, position ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;
        for row in subtask_rows {
            if let Some(&(project_position, task_position)) = task_index.get(&row.task_id) {
                projects[project_position].tasks[task_position]
                    .subtasks
                    .push(ExportSubtask {
                        id: row.id,
                        title: row.title,
                        done: row.done,
                        position: row.position,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    });
            }
        }

        let tag_rows: Vec<TagRow> =
            sqlx::query_as("SELECT id, name FROM tags ORDER BY name ASC, id ASC")
                .fetch_all(&self.pool)
                .await
                .map_err(unavailable)?;
        let tags = tag_rows
            .into_iter()
            .map(|row| ExportTag {
                id: row.id,
                name: row.name,
            })
            .collect();

        let task_tag_rows: Vec<TaskTagRow> = sqlx::query_as(
            "SELECT tt.task_id, t.name
             FROM task_tags tt
             JOIN tags t ON t.id = tt.tag_id
             ORDER BY tt.task_id ASC, t.name ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;
        for row in task_tag_rows {
            if let Some(&(project_position, task_position)) = task_index.get(&row.task_id) {
                projects[project_position].tasks[task_position]
                    .tags
                    .push(row.name);
            }
        }

        let comment_rows: Vec<CommentRow> = sqlx::query_as(
            "SELECT id, task_id, body, created_at, updated_at
             FROM comments
             ORDER BY task_id ASC, created_at ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;
        for row in comment_rows {
            if let Some(&(project_position, task_position)) = task_index.get(&row.task_id) {
                projects[project_position].tasks[task_position]
                    .comments
                    .push(ExportComment {
                        id: row.id,
                        body: row.body,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    });
            }
        }

        let attachment_rows: Vec<AttachmentRow> = sqlx::query_as(
            "SELECT id, task_id, filename, content_type, size, created_at
             FROM attachments
             ORDER BY task_id ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;
        for row in attachment_rows {
            if let Some(&(project_position, task_position)) = task_index.get(&row.task_id) {
                projects[project_position].tasks[task_position]
                    .attachments
                    .push(ExportAttachment {
                        id: row.id,
                        filename: row.filename,
                        content_type: row.content_type,
                        size: row.size,
                        created_at: row.created_at,
                    });
            }
        }

        Ok(ExportData { projects, tags })
    }
}
