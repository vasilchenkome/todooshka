/// Ошибки хранилища: разбор адреса, открытие пула, применение миграций.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("DATABASE_URL должен начинаться с sqlite://, получено {0:?}")]
    InvalidUrl(String),

    #[error("не удалось открыть базу: {0}")]
    Connect(#[source] sqlx::Error),

    #[error("миграции не применились: {0}")]
    Migrate(#[source] sqlx::migrate::MigrateError),

    #[error("копия базы не снята: {0}")]
    Backup(String),
}
