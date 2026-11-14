use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use todooshka_app::{CreateProject, UpdateProject};
use todooshka_domain::{ProjectId, TaskStatus};

use crate::dto::{
    CreateProjectRequest, ListQuery, ProjectDto, ProjectListEntryDto, StatusCounts,
    UpdateProjectRequest,
};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<ProjectListEntryDto>>, ApiError> {
    let projects = state.projects.list(query.include_archived).await?;
    let counts = state.tasks.counts_by_project().await?;

    let mut by_project: HashMap<i64, StatusCounts> = HashMap::new();
    for count in counts {
        let entry = by_project.entry(count.project_id.as_raw()).or_default();
        match count.status {
            TaskStatus::Backlog => entry.backlog = count.tasks,
            TaskStatus::InProgress => entry.in_progress = count.tasks,
            TaskStatus::Done => entry.done = count.tasks,
        }
    }

    let items = projects
        .iter()
        .map(|project| ProjectListEntryDto {
            project: ProjectDto::from(project),
            counts: by_project
                .remove(&project.id().as_raw())
                .unwrap_or_default(),
        })
        .collect();

    Ok(Json(items))
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

/// Deleting a project means archiving it: the tasks do not disappear.
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
