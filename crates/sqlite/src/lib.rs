//! Адаптер хранения: SQLite через sqlx, реализации портов и миграции.
//!
//! Здесь живут SQL, маппинг строк в доменные объекты и транзакции; бизнес-правил нет.

mod error;
mod mapping;
mod migrations;
mod pool;
mod projects;
mod tasks;

pub use error::StorageError;
pub use migrations::{migrate, migrator};
pub use pool::connect;
pub use projects::SqliteProjects;
pub use tasks::SqliteTasks;
