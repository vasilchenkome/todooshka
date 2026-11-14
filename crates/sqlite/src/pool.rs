use std::str::FromStr;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

use crate::error::StorageError;

/// Пул ограничен: один пользователь, лишние соединения только маскируют утечки.
const MAX_CONNECTIONS: u32 = 5;

/// Сколько ждать свободного соединения из пула, прежде чем вернуть ошибку.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Сколько SQLite ждёт снятия блокировки записи.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Открывает пул с настройками, которые нужны приложению: WAL, внешние ключи, таймауты.
pub async fn connect(database_url: &str) -> Result<SqlitePool, StorageError> {
    // Схему проверяем сами: sqlx молча трактует чужой URL как имя файла.
    if !database_url.starts_with("sqlite:") {
        return Err(StorageError::InvalidUrl(database_url.to_owned()));
    }

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|_| StorageError::InvalidUrl(database_url.to_owned()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(BUSY_TIMEOUT)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(StorageError::Connect)
}
