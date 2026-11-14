//! Ядро домена: сущности, инварианты и правила задач и проектов.
//!
//! Крейт намеренно не зависит от axum, sqlx, tokio, конфигурации и форматов API:
//! зависимости направлены только внутрь, к бизнес-правилам.

mod error;
mod project;
mod task;
mod time;

pub use error::DomainError;
pub use project::{
    DEFAULT_PROJECT_COLOR, MAX_PROJECT_DESCRIPTION_CHARS, MAX_PROJECT_NAME_CHARS, Project,
    ProjectColor, ProjectDraft, ProjectId, ProjectName, ProjectSnapshot,
};
pub use task::{TaskId, TaskStatus};
pub use time::Timestamp;
