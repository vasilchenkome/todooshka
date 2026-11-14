use axum::Json;
use axum::extract::{Path, State};
use todooshka_domain::{ProjectId, TaskId};

use crate::dto::{ReplaceTagsRequest, TagDto, TaskTagDto};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TagDto>>, ApiError> {
    let tags = state.tags.list_all().await?;
    Ok(Json(tags.iter().map(TagDto::from).collect()))
}

pub async fn list_by_project(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<TaskTagDto>>, ApiError> {
    let links = state
        .tags
        .list_by_project(ProjectId::from_raw(project_id))
        .await?;

    Ok(Json(links.iter().map(TaskTagDto::from).collect()))
}

pub async fn replace(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    Json(request): Json<ReplaceTagsRequest>,
) -> Result<Json<Vec<TagDto>>, ApiError> {
    let tags = state
        .tags
        .replace(TaskId::from_raw(task_id), &request.tags)
        .await?;

    Ok(Json(tags.iter().map(TagDto::from).collect()))
}
