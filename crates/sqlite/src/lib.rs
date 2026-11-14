//! Адаптер хранения: SQLite через sqlx, реализации портов и миграции.
//!
//! Здесь живут SQL, маппинг строк в доменные объекты и транзакции; бизнес-правил нет.

mod attachments;
mod backup;
mod comments;
mod error;
mod mapping;
mod migrations;
mod pool;
mod projects;
mod sessions;
mod subtasks;
mod tags;
mod tasks;
mod users;

pub use attachments::SqliteAttachments;
pub use backup::backup_to;
pub use comments::SqliteComments;
pub use error::StorageError;
pub use migrations::{migrate, migrator};
pub use pool::connect;
pub use projects::SqliteProjects;
pub use sessions::SqliteSessions;
pub use subtasks::SqliteSubtasks;
pub use tags::SqliteTags;
pub use tasks::SqliteTasks;
pub use users::SqliteUsers;
