use std::str::FromStr;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

use crate::error::StorageError;

/// The pool is capped: a single user, and spare connections only hide leaks.
const MAX_CONNECTIONS: u32 = 5;

/// How long to wait for a free pooled connection before returning an error.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long SQLite waits for a write lock to be released.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Opens the pool with the settings the app needs: WAL, foreign keys, timeouts.
pub async fn connect(database_url: &str) -> Result<SqlitePool, StorageError> {
    // We check the scheme ourselves: sqlx silently treats a foreign URL as a file name.
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
