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
