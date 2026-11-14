use todooshka_app::RepositoryError;

/// Driver error: the storage is unreachable (network, lock, disk).
pub(crate) fn unavailable(error: sqlx::Error) -> RepositoryError {
    RepositoryError::Unavailable(error.to_string())
}

/// Data from the DB breaks domain rules: that is corruption, not a storage failure.
pub(crate) fn corrupted(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Corrupted(error.to_string())
}
