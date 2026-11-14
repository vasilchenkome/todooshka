//! Сценарии приложения (use cases) и порты, которые они требуют от инфраструктуры.
//!
//! Один сценарий — одно действие пользователя. Крейт зависит только от домена:
//! конкретная БД, HTTP и чтение конфигурации живут снаружи.

mod error;
mod ports;
mod projects;
mod tasks;

pub use error::{ApplicationError, RepositoryError};
pub use ports::{Clock, ProjectRepository, TaskStore, TaskTransaction};
pub use projects::{CreateProject, ProjectService, UpdateProject};
pub use tasks::{AddTask, EditTask, MoveTask, TaskService};
