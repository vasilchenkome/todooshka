use sqlx::SqlitePool;
use sqlx::migrate::Migrator;

use crate::error::StorageError;

/// Migrations are embedded in the binary: the path resolves against the crate at compile time.
pub fn migrator() -> Migrator {
    sqlx::migrate!("./migrations")
}

/// Applies migrations at startup. Calling it twice is safe: sqlx tracks `_sqlx_migrations`
/// and applies only the missing files under a lock.
pub async fn migrate(pool: &SqlitePool) -> Result<(), StorageError> {
    migrator().run(pool).await.map_err(StorageError::Migrate)
}
