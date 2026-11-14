//! Ядро домена: сущности, инварианты и правила задач и проектов.
//!
//! Крейт намеренно не зависит от axum, sqlx, tokio, конфигурации и форматов API:
//! зависимости направлены только внутрь, к бизнес-правилам.

mod error;
mod ordering;
mod project;
mod task;
mod time;

pub use error::DomainError;
pub use ordering::{STEP, between, evenly};
pub use project::{
    DEFAULT_PROJECT_COLOR, MAX_PROJECT_DESCRIPTION_CHARS, MAX_PROJECT_NAME_CHARS, Project,
    ProjectColor, ProjectDraft, ProjectId, ProjectName, ProjectSnapshot,
};
pub use task::{
    MAX_TASK_DESCRIPTION_CHARS, MAX_TASK_TITLE_CHARS, Task, TaskDraft, TaskId, TaskSnapshot,
    TaskStatus, TaskTitle,
};
pub use time::Timestamp;
