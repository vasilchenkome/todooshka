use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use todooshka_domain::{CommentId, ProjectId, TaskId};

use crate::dto::{CommentDto, CommentRequest};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<CommentDto>>, ApiError> {
    let comments = state.comments.list(ProjectId::from_raw(project_id)).await?;

    Ok(Json(comments.iter().map(CommentDto::from).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    Json(request): Json<CommentRequest>,
) -> Result<(StatusCode, Json<CommentDto>), ApiError> {
    let comment = state
        .comments
        .add(TaskId::from_raw(task_id), &request.body)
        .await?;

    Ok((StatusCode::CREATED, Json(CommentDto::from(&comment))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<CommentRequest>,
) -> Result<Json<CommentDto>, ApiError> {
    let comment = state
        .comments
        .edit(CommentId::from_raw(id), &request.body)
        .await?;

    Ok(Json(CommentDto::from(&comment)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.comments.delete(CommentId::from_raw(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
