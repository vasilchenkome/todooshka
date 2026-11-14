//! Сценарии приложения (use cases) и порты, которые они требуют от инфраструктуры.
//!
//! Один сценарий — одно действие пользователя. Крейт зависит только от домена:
//! конкретная БД, HTTP и чтение конфигурации живут снаружи.

mod auth;
mod error;
mod ports;
mod projects;
mod tasks;

pub use auth::{
    AuthService, AuthenticatedUser, ChangePassword, CreateUser, IssuedSession, Login,
    SESSION_TOUCH_SECONDS, SESSION_TTL_SECONDS,
};
pub use error::{ApplicationError, RepositoryError};
pub use ports::{
    Clock, PasswordHasher, ProjectRepository, SessionStore, SessionTokens, StoredSession,
    TaskStore, TaskTransaction, UserStore,
};
pub use projects::{CreateProject, ProjectService, UpdateProject};
pub use tasks::{AddTask, EditTask, MoveTask, TaskService};
