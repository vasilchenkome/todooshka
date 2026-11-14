//! Адаптер хранения: SQLite через sqlx, реализации портов и миграции.
//!
//! Здесь живут SQL, маппинг строк в доменные объекты и транзакции; бизнес-правил нет.

mod error;
mod migrations;
mod pool;

pub use error::StorageError;
pub use migrations::{migrate, migrator};
pub use pool::connect;
