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

    #[error("статус задачи и время завершения не согласованы")]
    TaskCompletionInconsistent,
}
