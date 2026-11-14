use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use todooshka_domain::{AttachmentId, ProjectId, TaskId};

use crate::dto::AttachmentDto;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<AttachmentDto>>, ApiError> {
    let files = state
        .attachments
        .list(ProjectId::from_raw(project_id))
        .await?;

    Ok(Json(files.iter().map(AttachmentDto::from).collect()))
}

pub async fn upload(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<AttachmentDto>), ApiError> {
    let mut uploaded = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::validation(format!("не удалось прочитать файл: {error}")))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field.file_name().unwrap_or_default().to_owned();
        let data = field
            .bytes()
            .await
            .map_err(|error| ApiError::validation(format!("не удалось прочитать файл: {error}")))?;
        uploaded = Some((filename, data.to_vec()));
        break;
    }

    let (filename, data) =
        uploaded.ok_or_else(|| ApiError::validation("прикрепите файл в поле file"))?;

    let meta = state
        .attachments
        .add(TaskId::from_raw(task_id), &filename, data)
        .await?;

    Ok((StatusCode::CREATED, Json(AttachmentDto::from(&meta))))
}

pub async fn download(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let file = state.attachments.get(AttachmentId::from_raw(id)).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&file.meta.content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename*=UTF-8''{}",
            encode_filename(file.meta.filename.as_str())
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    Ok((headers, file.data).into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.attachments.delete(AttachmentId::from_raw(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn encode_filename(name: &str) -> String {
    name.bytes()
        .map(|byte| {
            let allowed =
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b' ');
            if allowed {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}
