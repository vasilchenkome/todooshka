//! Application use cases and the ports they require from the infrastructure.
//!
//! One use case, one user action. The crate depends only on the domain:
//! a concrete DB, HTTP and config reading all live outside.

mod agenda;
mod attachments;
mod auth;
mod comments;
mod error;
mod export;
mod ports;
mod projects;
mod subtasks;
mod tags;
mod tasks;

pub use agenda::{Agenda, AgendaEntry, AgendaService};
pub use attachments::AttachmentService;
pub use auth::{
    AuthService, AuthenticatedUser, ChangePassword, CreateUser, IssuedSession, Login,
    SESSION_TOUCH_SECONDS, SESSION_TTL_SECONDS,
};
pub use comments::CommentService;
pub use error::{ApplicationError, RepositoryError};
pub use export::{
    ExportAttachment, ExportComment, ExportData, ExportProject, ExportService, ExportSnapshot,
    ExportSubtask, ExportTag, ExportTask,
};
pub use ports::{
    AgendaReader, AttachmentStore, Clock, CommentStore, ExportReader, PasswordHasher,
    ProjectRepository, ProjectTaskCount, SessionStore, SessionTokens, StoredSession, SubtaskStore,
    TagStore, TaskStore, TaskTransaction, UserStore,
};
pub use projects::{CreateProject, ProjectService, UpdateProject};
pub use subtasks::SubtaskService;
pub use tags::TagService;
pub use tasks::{AddTask, EditTask, MoveTask, TaskService};
