//! Storage adapter: SQLite through sqlx, port implementations and migrations.
//!
//! SQL, row-to-domain mapping and transactions live here; no business rules.

mod agenda;
mod attachments;
mod backup;
mod comments;
mod error;
mod export;
mod mapping;
mod migrations;
mod pool;
mod projects;
mod sessions;
mod subtasks;
mod tags;
mod tasks;
mod users;

pub use agenda::SqliteAgenda;
pub use attachments::SqliteAttachments;
pub use backup::backup_to;
pub use comments::SqliteComments;
pub use error::StorageError;
pub use export::SqliteExport;
pub use migrations::{migrate, migrator};
pub use pool::connect;
pub use projects::SqliteProjects;
pub use sessions::SqliteSessions;
pub use subtasks::SqliteSubtasks;
pub use tags::SqliteTags;
pub use tasks::SqliteTasks;
pub use users::SqliteUsers;
