use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use todooshka_domain::{ProjectId, SubtaskId, TaskId};

use crate::dto::{CreateSubtaskRequest, EditSubtaskRequest, SubtaskDto};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<SubtaskDto>>, ApiError> {
    let items = state.subtasks.list(ProjectId::from_raw(project_id)).await?;

    Ok(Json(items.iter().map(SubtaskDto::from).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    Json(request): Json<CreateSubtaskRequest>,
) -> Result<(StatusCode, Json<SubtaskDto>), ApiError> {
    let subtask = state
        .subtasks
        .add(TaskId::from_raw(task_id), &request.title)
        .await?;

    Ok((StatusCode::CREATED, Json(SubtaskDto::from(&subtask))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<EditSubtaskRequest>,
) -> Result<Json<SubtaskDto>, ApiError> {
    let subtask = state
        .subtasks
        .edit(
            SubtaskId::from_raw(id),
            request.title.as_deref(),
            request.done,
        )
        .await?;

    Ok(Json(SubtaskDto::from(&subtask)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.subtasks.delete(SubtaskId::from_raw(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
