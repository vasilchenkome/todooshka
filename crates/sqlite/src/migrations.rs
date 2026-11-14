use sqlx::SqlitePool;
use sqlx::migrate::Migrator;

use crate::error::StorageError;

/// Миграции вшиты в бинарь: путь разрешается относительно крейта на этапе компиляции.
pub fn migrator() -> Migrator {
    sqlx::migrate!("./migrations")
}

/// Применяет миграции на старте. Повторный вызов безопасен: sqlx ведёт `_sqlx_migrations`
/// и под блокировкой применяет только недостающие файлы.
pub async fn migrate(pool: &SqlitePool) -> Result<(), StorageError> {
    migrator().run(pool).await.map_err(StorageError::Migrate)
}
