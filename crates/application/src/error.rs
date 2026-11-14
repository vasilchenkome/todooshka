use todooshka_domain::{
    AttachmentId, CommentId, DomainError, ProjectId, SubtaskId, TaskId, Username,
};

/// Как порт хранения сообщает о своих проблемах, не выдавая драйвер БД.
///
/// Тексты ошибок здесь — для логов и диагностики; пользователю HTTP-слой покажет
/// обобщённое сообщение.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RepositoryError {
    #[error("хранилище недоступно: {0}")]
    Unavailable(String),

    #[error("данные в хранилище расходятся с доменными правилами: {0}")]
    Corrupted(String),

    #[error("конфликт записи: {0}")]
    Conflict(String),
}

/// Ошибки сценариев: доменные правила, отсутствие сущности, отказ хранилища.
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
