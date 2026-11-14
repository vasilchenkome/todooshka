//! Сценарии приложения (use cases) и порты, которые они требуют от инфраструктуры.
//!
//! Один сценарий — одно действие пользователя. Крейт зависит только от домена:
//! конкретная БД, HTTP и чтение конфигурации живут снаружи.

mod error;
mod ports;
mod projects;

pub use error::{ApplicationError, RepositoryError};
pub use ports::{Clock, ProjectRepository};
pub use projects::{CreateProject, ProjectService, RenameProject};
