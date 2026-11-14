use todooshka_domain::{
    AttachmentId, CommentId, DomainError, ProjectId, SubtaskId, TaskId, Username,
};

/// How a storage port reports its own problems without exposing the DB driver.
///
/// Error texts here are for logs and diagnostics; the HTTP layer shows the user
/// a generic message.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RepositoryError {
    #[error("хранилище недоступно: {0}")]
    Unavailable(String),

    #[error("данные в хранилище расходятся с доменными правилами: {0}")]
    Corrupted(String),

    #[error("конфликт записи: {0}")]
    Conflict(String),
}

/// Use-case errors: domain rules, a missing entity, a storage refusal.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error("проект {0} не найден")]
    ProjectNotFound(ProjectId),

    #[error("задача {0} не найдена")]
    TaskNotFound(TaskId),

    #[error("пункт чек-листа {0} не найден")]
    SubtaskNotFound(SubtaskId),

    #[error("комментарий {0} не найден")]
    CommentNotFound(CommentId),

    #[error("вложение {0} не найдено")]
    AttachmentNotFound(AttachmentId),

    #[error("к задаче уже прикреплено 10 файлов")]
    AttachmentsLimitReached,

    #[error("неверный логин или пароль")]
    InvalidCredentials,

    #[error("пользователь {0} не найден")]
    UserNotFound(Username),

    #[error("логин {0} уже занят")]
    UsernameTaken(Username),
}
