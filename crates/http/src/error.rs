use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use todooshka_app::{ApplicationError, RepositoryError};
use todooshka_domain::DomainError;

use crate::locale::{self, ApiLocale};

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message_ru: String,
    message_en: String,
}

impl ApiError {
    fn new(
        status: StatusCode,
        code: &'static str,
        message_ru: impl Into<String>,
        message_en: impl Into<String>,
    ) -> Self {
        Self {
            status,
            code,
            message_ru: message_ru.into(),
            message_en: message_en.into(),
        }
    }

    pub fn localized(
        status: StatusCode,
        code: &'static str,
        message_ru: impl Into<String>,
        message_en: impl Into<String>,
    ) -> Self {
        Self::new(status, code, message_ru, message_en)
    }

    pub fn bad_request(code: &'static str, ru: impl Into<String>, en: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, ru, en)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message.clone(),
            message,
        )
    }

    pub fn validation_localized(ru: impl Into<String>, en: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            ru,
            en,
        )
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(StatusCode::NOT_FOUND, "not_found", message.clone(), message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            message.clone(),
            message,
        )
    }

    pub fn too_many_attempts() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_attempts",
            "слишком много попыток входа, попробуйте через минуту",
            "too many sign-in attempts, try again in a minute",
        )
    }

    fn message(&self) -> &str {
        match locale::current() {
            ApiLocale::Ru => &self.message_ru,
            ApiLocale::En => &self.message_en,
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        let message_ru = error.to_string();
        let message_en = domain_message_en(&error);
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message_ru,
            message_en,
        )
    }
}

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        match error {
            ApplicationError::Domain(DomainError::AttachmentTooLarge) => Self::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "файл больше 10 МБ",
                "file is larger than 10 MB",
            ),
            ApplicationError::Domain(err) => Self::from(err),
            ApplicationError::ProjectNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("проект {id} не найден"),
                format!("project {id} not found"),
            ),
            ApplicationError::TaskNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("задача {id} не найдена"),
                format!("task {id} not found"),
            ),
            ApplicationError::SubtaskNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("пункт чек-листа {id} не найден"),
                format!("checklist item {id} not found"),
            ),
            ApplicationError::CommentNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("комментарий {id} не найден"),
                format!("comment {id} not found"),
            ),
            ApplicationError::AttachmentNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("вложение {id} не найдено"),
                format!("attachment {id} not found"),
            ),
            ApplicationError::AttachmentsLimitReached => Self::new(
                StatusCode::CONFLICT,
                "attachments_limit",
                "к задаче уже прикреплено 10 файлов",
                "a task can have at most 10 attachments",
            ),
            ApplicationError::UserNotFound(username) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("пользователь {username} не найден"),
                format!("user {username} not found"),
            ),
            ApplicationError::UsernameTaken(username) => Self::new(
                StatusCode::CONFLICT,
                "username_taken",
                format!("логин {username} уже занят"),
                format!("username {username} is already taken"),
            ),
            ApplicationError::InvalidCredentials => Self::new(
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "неверный логин или пароль",
                "wrong username or password",
            ),
            ApplicationError::Repository(RepositoryError::Conflict(message)) => Self::new(
                StatusCode::CONFLICT,
                "write_conflict",
                message,
                "write conflict, reload the data and try again",
            ),
            ApplicationError::Repository(RepositoryError::Unavailable(detail)) => {
                tracing::error!(%detail, "хранилище недоступно");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_unavailable",
                    "хранилище недоступно",
                    "storage is unavailable",
                )
            }
            ApplicationError::Repository(RepositoryError::Corrupted(detail)) => {
                tracing::error!(%detail, "данные в хранилище испорчены");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_corrupted",
                    "данные повреждены",
                    "stored data is corrupted",
                )
            }
        }
    }
}

fn domain_message_en(error: &DomainError) -> String {
    match error {
        DomainError::EmptyProjectName => "project name cannot be empty".to_owned(),
        DomainError::ProjectNameTooLong { max } => {
            format!("project name is longer than {max} characters")
        }
        DomainError::ProjectDescriptionTooLong { max } => {
            format!("project description is longer than {max} characters")
        }
        DomainError::InvalidProjectColor(value) => {
            format!("color must be in #RRGGBB format, got {value:?}")
        }
        DomainError::EmptyTaskTitle => "task title cannot be empty".to_owned(),
        DomainError::TaskTitleTooLong { max } => {
            format!("task title is longer than {max} characters")
        }
        DomainError::TaskDescriptionTooLong { max } => {
            format!("task description is longer than {max} characters")
        }
        DomainError::InvalidTaskPosition => "task position must be a finite number".to_owned(),
        DomainError::InvalidDueDate(value) => {
            format!("due date must be a real date in YYYY-MM-DD format, got {value:?}")
        }
        DomainError::InvalidEstimateMinutes => {
            "estimate must be between 1 and 100000 minutes".to_owned()
        }
        DomainError::InvalidPriority(value) => format!("unknown priority: {value:?}"),
        DomainError::EmptySubtaskTitle => "checklist item cannot be empty".to_owned(),
        DomainError::SubtaskTitleTooLong { max } => {
            format!("checklist item is longer than {max} characters")
        }
        DomainError::EmptyTag => "tag cannot be empty".to_owned(),
        DomainError::TagTooLong { max } => format!("tag is longer than {max} characters"),
        DomainError::EmptyComment => "comment cannot be empty".to_owned(),
        DomainError::CommentTooLong { max } => {
            format!("comment is longer than {max} characters")
        }
        DomainError::EmptyAttachmentFilename => "file name cannot be empty".to_owned(),
        DomainError::AttachmentFilenameTooLong => {
            "file name is longer than 255 characters".to_owned()
        }
        DomainError::InvalidAttachmentFilename => {
            "file name contains control characters".to_owned()
        }
        DomainError::AttachmentTooLarge => "file is larger than 10 MB".to_owned(),
        DomainError::EmptyAttachment => "file is empty".to_owned(),
        DomainError::TaskCompletionInconsistent => {
            "task status and completion time are inconsistent".to_owned()
        }
        DomainError::EmptyUsername => "username cannot be empty".to_owned(),
        DomainError::UsernameTooLong { max } => {
            format!("username is longer than {max} characters")
        }
        DomainError::InvalidUsername => "username contains invalid characters".to_owned(),
        DomainError::WeakPassword { min } => {
            format!("password must be at least {min} characters")
        }
        DomainError::InvalidPasswordHash => "password hash is empty".to_owned(),
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message().to_owned(),
            },
        };
        (self.status, Json(body)).into_response()
    }
}
