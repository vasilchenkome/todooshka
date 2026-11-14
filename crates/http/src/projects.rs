use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use todooshka_app::{CreateProject, UpdateProject};
use todooshka_domain::ProjectId;

use crate::dto::{CreateProjectRequest, ListQuery, ProjectDto, UpdateProjectRequest};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<ProjectDto>>, ApiError> {
    let projects = state.projects.list(query.include_archived).await?;
    let projects = projects.iter().map(ProjectDto::from).collect();
    Ok(Json(projects))
}

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectDto>), ApiError> {
    let project = state
        .projects
        .create(CreateProject {
            name: request.name,
            description: request.description,
            color: request.color,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(ProjectDto::from(&project))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectDto>, ApiError> {
    let project = state
        .projects
        .update(UpdateProject {
            id: ProjectId::from_raw(id),
            name: request.name,
            description: request.description,
            color: request.color,
            archived: request.archived,
        })
        .await?;

    Ok(Json(ProjectDto::from(&project)))
}

/// Удаление проекта = архивирование: задачи не пропадают.
pub async fn archive(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state
        .projects
        .update(UpdateProject {
            id: ProjectId::from_raw(id),
            name: None,
            description: None,
            color: None,
            archived: Some(true),
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
