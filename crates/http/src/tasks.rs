use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use todooshka_app::{AddTask, EditTask, MoveTask};
use todooshka_domain::{ProjectId, TaskId, TaskStatus};

use crate::dto::{CreateTaskRequest, EditTaskRequest, MoveTaskRequest, TaskDto};
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
) -> Result<Json<Vec<TaskDto>>, ApiError> {
    let tasks = state.tasks.list(ProjectId::from_raw(project_id)).await?;

    Ok(Json(tasks.iter().map(TaskDto::from).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskDto>), ApiError> {
    let status = match request.status.as_deref() {
        None => TaskStatus::Backlog,
        Some(code) => parse_status(code)?,
    };

    let task = state
        .tasks
        .add(AddTask {
            project_id: ProjectId::from_raw(project_id),
            title: request.title,
            description: request.description,
            status,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(TaskDto::from(&task))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<EditTaskRequest>,
) -> Result<Json<TaskDto>, ApiError> {
    let task = state
        .tasks
        .edit(EditTask {
            id: TaskId::from_raw(id),
            title: request.title,
            description: request.description,
        })
        .await?;

    Ok(Json(TaskDto::from(&task)))
}

pub async fn move_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<MoveTaskRequest>,
) -> Result<Json<TaskDto>, ApiError> {
    let task = state
        .tasks
        .move_task(MoveTask {
            id: TaskId::from_raw(id),
            status: parse_status(&request.status)?,
            target_index: request.target_index,
        })
        .await?;

    Ok(Json(TaskDto::from(&task)))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.tasks.delete(TaskId::from_raw(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Коды статусов — часть контракта API; всё остальное считаем ошибкой формата.
fn parse_status(code: &str) -> Result<TaskStatus, ApiError> {
    TaskStatus::from_code(code)
        .ok_or_else(|| ApiError::validation(format!("неизвестный статус задачи {code:?}")))
}
