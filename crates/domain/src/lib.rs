//! Domain core: entities, invariants and the rules for tasks and projects.
//!
//! The crate deliberately ignores axum, sqlx, tokio, config and API formats:
//! dependencies point inward only, towards the business rules.

mod attachment;
mod comment;
mod error;
mod ordering;
mod planning;
mod project;
mod subtask;
mod tag;
mod task;
mod time;
mod user;

pub use attachment::{
    AttachmentFilename, AttachmentId, AttachmentMeta, MAX_ATTACHMENT_BYTES,
    MAX_ATTACHMENTS_PER_TASK, content_type_for,
};
pub use comment::{Comment, CommentBody, CommentId, CommentSnapshot, MAX_COMMENT_CHARS};
pub use error::DomainError;
pub use ordering::{STEP, between, evenly};
pub use planning::{DueDate, EstimateMinutes, MAX_ESTIMATE_MINUTES, Planning, Priority};
pub use project::{
    DEFAULT_PROJECT_COLOR, MAX_PROJECT_DESCRIPTION_CHARS, MAX_PROJECT_NAME_CHARS, Project,
    ProjectColor, ProjectDraft, ProjectId, ProjectName, ProjectSnapshot,
};
pub use subtask::{MAX_SUBTASK_TITLE_CHARS, Subtask, SubtaskId, SubtaskSnapshot, SubtaskTitle};
pub use tag::{MAX_TAG_CHARS, Tag, TagId, TagName, TaskTag};
pub use task::{
    MAX_TASK_DESCRIPTION_CHARS, MAX_TASK_TITLE_CHARS, Task, TaskDraft, TaskId, TaskSnapshot,
    TaskStatus, TaskTitle,
};
pub use time::Timestamp;
pub use user::{
    MAX_USERNAME_CHARS, MIN_PASSWORD_CHARS, PasswordHash, User, UserId, Username, validate_password,
};
