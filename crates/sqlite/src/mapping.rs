use todooshka_app::RepositoryError;

/// Ошибка драйвера: хранилище недоступно (сеть, блокировка, диск).
pub(crate) fn unavailable(error: sqlx::Error) -> RepositoryError {
    RepositoryError::Unavailable(error.to_string())
}

/// Данные из БД не проходят доменные правила: это порча, а не отказ хранилища.
pub(crate) fn corrupted(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Corrupted(error.to_string())
}
