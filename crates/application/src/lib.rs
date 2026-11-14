//! Application use cases and the ports they require from the infrastructure.
//!
//! One use case, one user action. The crate depends only on the domain:
//! a concrete DB, HTTP and config reading all live outside.

mod attachments;
mod auth;
mod comments;
mod error;
mod ports;
mod projects;
mod subtasks;
mod tags;
mod tasks;

pub use attachments::AttachmentService;
pub use auth::{
    AuthService, AuthenticatedUser, ChangePassword, CreateUser, IssuedSession, Login,
    SESSION_TOUCH_SECONDS, SESSION_TTL_SECONDS,
};
pub use comments::CommentService;
pub use error::{ApplicationError, RepositoryError};
pub use ports::{
    AttachmentStore, Clock, CommentStore, PasswordHasher, ProjectRepository, ProjectTaskCount,
    SessionStore, SessionTokens, StoredSession, SubtaskStore, TagStore, TaskStore, TaskTransaction,
    UserStore,
};
pub use projects::{CreateProject, ProjectService, UpdateProject};
pub use subtasks::SubtaskService;
pub use tags::TagService;
pub use tasks::{AddTask, EditTask, MoveTask, TaskService};
