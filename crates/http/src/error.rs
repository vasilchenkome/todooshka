use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use todooshka_app::{ApplicationError, RepositoryError};

/// Ошибка API: один формат тела на все неуспехи, коды отличаются статусом и `code`.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    /// Ошибка разбора запроса: та же форма, что у доменной валидации.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        )
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", message)
    }

    pub fn too_many_attempts() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_attempts",
            "слишком много попыток входа, попробуйте через минуту",
        )
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

impl From<todooshka_domain::DomainError> for ApiError {
    fn from(error: todooshka_domain::DomainError) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        )
    }
}

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        match error {
            ApplicationError::Domain(err) => Self::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                err.to_string(),
            ),
            ApplicationError::ProjectNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("проект {id} не найден"),
            ),
            ApplicationError::TaskNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("задача {id} не найдена"),
            ),
            ApplicationError::SubtaskNotFound(id) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("пункт чек-листа {id} не найден"),
            ),
            ApplicationError::UserNotFound(username) => Self::new(
                StatusCode::NOT_FOUND,
                "not_found",
                format!("пользователь {username} не найден"),
            ),
            ApplicationError::UsernameTaken(username) => Self::new(
                StatusCode::CONFLICT,
                "username_taken",
                format!("логин {username} уже занят"),
            ),
            ApplicationError::InvalidCredentials => Self::new(
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "неверный логин или пароль",
            ),
            ApplicationError::Repository(RepositoryError::Conflict(message)) => {
                Self::new(StatusCode::CONFLICT, "write_conflict", message)
            }
            // Детали хранилища — в лог, наружу только безопасная формулировка.
            ApplicationError::Repository(RepositoryError::Unavailable(detail)) => {
                tracing::error!(%detail, "хранилище недоступно");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_unavailable",
                    "хранилище недоступно",
                )
            }
            ApplicationError::Repository(RepositoryError::Corrupted(detail)) => {
                tracing::error!(%detail, "данные в хранилище испорчены");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "storage_corrupted",
                    "данные повреждены",
                )
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, Json(body)).into_response()
    }
}
