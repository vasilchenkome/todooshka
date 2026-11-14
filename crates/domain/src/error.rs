/// Ошибки бизнес-правил. Как их показывать — решает HTTP-слой, не домен.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("название проекта не может быть пустым")]
    EmptyProjectName,

    #[error("название проекта длиннее {max} символов")]
    ProjectNameTooLong { max: usize },

    #[error("описание проекта длиннее {max} символов")]
    ProjectDescriptionTooLong { max: usize },

    #[error("цвет должен быть в формате #RRGGBB, получено {0:?}")]
    InvalidProjectColor(String),

    #[error("заголовок задачи не может быть пустым")]
    EmptyTaskTitle,

    #[error("заголовок задачи длиннее {max} символов")]
    TaskTitleTooLong { max: usize },

    #[error("описание задачи длиннее {max} символов")]
    TaskDescriptionTooLong { max: usize },

    #[error("позиция задачи должна быть конечным числом")]
    InvalidTaskPosition,

    #[error("срок должен быть реальной датой в формате ГГГГ-ММ-ДД, получено {0:?}")]
    InvalidDueDate(String),

    #[error("оценка должна быть от 1 до 100000 минут")]
    InvalidEstimateMinutes,

    #[error("неизвестный приоритет: {0:?}")]
    InvalidPriority(String),

    #[error("пункт чек-листа не может быть пустым")]
    EmptySubtaskTitle,

    #[error("пункт чек-листа длиннее {max} символов")]
    SubtaskTitleTooLong { max: usize },

    #[error("тег не может быть пустым")]
    EmptyTag,

    #[error("тег длиннее {max} символов")]
    TagTooLong { max: usize },

    #[error("комментарий не может быть пустым")]
    EmptyComment,

    #[error("комментарий длиннее {max} символов")]
    CommentTooLong { max: usize },

    #[error("имя файла не может быть пустым")]
    EmptyAttachmentFilename,

    #[error("имя файла длиннее 255 символов")]
    AttachmentFilenameTooLong,

    #[error("имя файла содержит управляющие символы")]
    InvalidAttachmentFilename,

    #[error("файл больше 10 МБ")]
    AttachmentTooLarge,

    #[error("файл пустой")]
    EmptyAttachment,

    #[error("статус задачи и время завершения не согласованы")]
    TaskCompletionInconsistent,

    #[error("логин не может быть пустым")]
    EmptyUsername,

    #[error("логин длиннее {max} символов")]
    UsernameTooLong { max: usize },

    #[error("логин содержит недопустимые символы")]
    InvalidUsername,

    #[error("пароль должен быть не короче {min} символов")]
    WeakPassword { min: usize },

    #[error("хеш пароля пуст")]
    InvalidPasswordHash,
}
