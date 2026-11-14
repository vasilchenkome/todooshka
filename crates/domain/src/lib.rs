//! Ядро домена: сущности, инварианты и правила задач и проектов.
//!
//! Крейт намеренно не зависит от axum, sqlx, tokio, конфигурации и форматов API:
//! зависимости направлены только внутрь, к бизнес-правилам.

mod error;
mod ordering;
mod planning;
mod project;
mod task;
mod time;
mod user;

pub use error::DomainError;
pub use ordering::{STEP, between, evenly};
pub use planning::{DueDate, EstimateMinutes, MAX_ESTIMATE_MINUTES, Planning, Priority};
pub use project::{
    DEFAULT_PROJECT_COLOR, MAX_PROJECT_DESCRIPTION_CHARS, MAX_PROJECT_NAME_CHARS, Project,
    ProjectColor, ProjectDraft, ProjectId, ProjectName, ProjectSnapshot,
};
pub use task::{
    MAX_TASK_DESCRIPTION_CHARS, MAX_TASK_TITLE_CHARS, Task, TaskDraft, TaskId, TaskSnapshot,
    TaskStatus, TaskTitle,
};
pub use time::Timestamp;
pub use user::{
    MAX_USERNAME_CHARS, MIN_PASSWORD_CHARS, PasswordHash, User, UserId, Username, validate_password,
};
