use std::sync::Arc;

use serde::Serialize;

use crate::error::ApplicationError;
use crate::ports::{Clock, ExportReader};

/// Versioned plain snapshot: no users, sessions or attachment bytes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportSnapshot {
    pub format: &'static str,
    pub version: u32,
    pub exported_at: String,
    pub projects: Vec<ExportProject>,
    pub tags: Vec<ExportTag>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportProject {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub color: String,
    pub position: i32,
    pub archived: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub tasks: Vec<ExportTask>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportTask {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: String,
    pub status: String,
    pub position: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    pub due_on: Option<String>,
    pub priority: String,
    pub estimate_minutes: Option<i64>,
    pub subtasks: Vec<ExportSubtask>,
    pub tags: Vec<String>,
    pub comments: Vec<ExportComment>,
    pub attachments: Vec<ExportAttachment>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportSubtask {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub position: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportComment {
    pub id: i64,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportAttachment {
    pub id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub created_at: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ExportTag {
    pub id: i64,
    pub name: String,
}

/// The reader assembles the tree; the use case stamps format and time.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportData {
    pub projects: Vec<ExportProject>,
    pub tags: Vec<ExportTag>,
}

pub struct ExportService {
    reader: Arc<dyn ExportReader>,
    clock: Arc<dyn Clock>,
}

impl ExportService {
    pub fn new(reader: Arc<dyn ExportReader>, clock: Arc<dyn Clock>) -> Self {
        Self { reader, clock }
    }

    pub async fn export(&self) -> Result<ExportSnapshot, ApplicationError> {
        let data = self.reader.read().await?;
        Ok(ExportSnapshot {
            format: "todooshka.export",
            version: 1,
            exported_at: self.clock.now().as_rfc3339(),
            projects: data.projects,
            tags: data.tags,
        })
    }
}
