//! Адаптер хранения: SQLite через sqlx, реализации портов и миграции.
//!
//! Здесь живут SQL, маппинг строк в доменные объекты и транзакции; бизнес-правил нет.

mod backup;
mod error;
mod mapping;
mod migrations;
mod pool;
mod projects;
mod sessions;
mod tasks;
mod users;

pub use backup::backup_to;
pub use error::StorageError;
pub use migrations::{migrate, migrator};
pub use pool::connect;
pub use projects::SqliteProjects;
pub use sessions::SqliteSessions;
pub use tasks::SqliteTasks;
pub use users::SqliteUsers;
