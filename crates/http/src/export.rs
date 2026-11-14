use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;

use crate::error::ApiError;
use crate::state::AppState;

/// Download a portable JSON snapshot. Cookies authenticate the request, so a plain link works.
pub async fn get(State(state): State<AppState>) -> Result<Response, ApiError> {
    let snapshot = state.export.export().await?;
    let body = serde_json::to_vec_pretty(&snapshot).map_err(|error| {
        tracing::error!(%error, "не удалось сериализовать экспорт");
        ApiError::localized(
            StatusCode::INTERNAL_SERVER_ERROR,
            "export_failed",
            "не удалось собрать экспорт",
            "could not build the export",
        )
    })?;

    let date = snapshot
        .exported_at
        .get(..10)
        .unwrap_or("export")
        .to_owned();
    let filename = format!("todooshka-{date}.json");

    let mut response = Response::new(Body::from(body));
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).map_err(|_| {
            ApiError::localized(
                StatusCode::INTERNAL_SERVER_ERROR,
                "export_failed",
                "не удалось собрать экспорт",
                "could not build the export",
            )
        })?,
    );

    Ok(response)
}
