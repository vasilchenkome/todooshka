use serde::{Deserialize, Serialize};
use todooshka_domain::{DueDate, EstimateMinutes, Planning, Priority, Project, Task};

/// Проект на проводе: плоские поля, время в unix-секундах. Домен сюда не протекает.
#[derive(Debug, Serialize)]
pub struct ProjectDto {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub color: String,
    pub position: i32,
    pub archived: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<&Project> for ProjectDto {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id().as_raw(),
            name: project.name().as_str().to_owned(),
            description: project.description().to_owned(),
            color: project.color().as_str().to_owned(),
            position: project.position(),
            archived: project.is_archived(),
            created_at: project.created_at().as_unix_seconds(),
            updated_at: project.updated_at().as_unix_seconds(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct StatusCounts {
    pub backlog: i64,
    pub in_progress: i64,
    pub done: i64,
}

/// Строка списка проектов: поля проекта и счётчики задач по колонкам.
#[derive(Debug, Serialize)]
pub struct ProjectListEntryDto {
    #[serde(flatten)]
    pub project: ProjectDto,
    pub counts: StatusCounts,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// PATCH: отсутствующее поле не меняется, `null` для строк — ошибка формата.
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub include_archived: bool,
}

/// Задача на проводе: статус — строковый код, позиция — дробное число колонки.
#[derive(Debug, Serialize)]
pub struct TaskDto {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: String,
    pub status: &'static str,
    pub position: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    pub due_on: Option<String>,
    pub priority: &'static str,
    pub estimate_minutes: Option<i64>,
}

impl From<&Task> for TaskDto {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id().as_raw(),
            project_id: task.project_id().as_raw(),
            title: task.title().as_str().to_owned(),
            description: task.description().to_owned(),
            status: task.status().as_code(),
            position: task.position(),
            created_at: task.created_at().as_unix_seconds(),
            updated_at: task.updated_at().as_unix_seconds(),
            completed_at: task.completed_at().map(|time| time.as_unix_seconds()),
            due_on: task
                .planning()
                .due_on()
                .map(|date| date.as_str().to_owned()),
            priority: task.planning().priority().as_code(),
            estimate_minutes: task.planning().estimate().map(EstimateMinutes::as_minutes),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubtaskDto {
    pub id: i64,
    pub task_id: i64,
    pub title: String,
    pub done: bool,
    pub position: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<&todooshka_domain::Subtask> for SubtaskDto {
    fn from(subtask: &todooshka_domain::Subtask) -> Self {
        Self {
            id: subtask.id().as_raw(),
            task_id: subtask.task_id().as_raw(),
            title: subtask.title().as_str().to_owned(),
            done: subtask.is_done(),
            position: subtask.position(),
            created_at: subtask.created_at().as_unix_seconds(),
            updated_at: subtask.updated_at().as_unix_seconds(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TagDto {
    pub id: i64,
    pub name: String,
}

impl From<&todooshka_domain::Tag> for TagDto {
    fn from(tag: &todooshka_domain::Tag) -> Self {
        Self {
            id: tag.id().as_raw(),
            name: tag.name().as_str().to_owned(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TaskTagDto {
    pub task_id: i64,
    pub name: String,
}

impl From<&todooshka_domain::TaskTag> for TaskTagDto {
    fn from(link: &todooshka_domain::TaskTag) -> Self {
        Self {
            task_id: link.task_id.as_raw(),
            name: link.name.as_str().to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReplaceTagsRequest {
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CommentDto {
    pub id: i64,
    pub task_id: i64,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<&todooshka_domain::Comment> for CommentDto {
    fn from(comment: &todooshka_domain::Comment) -> Self {
        Self {
            id: comment.id().as_raw(),
            task_id: comment.task_id().as_raw(),
            body: comment.body().as_str().to_owned(),
            created_at: comment.created_at().as_unix_seconds(),
            updated_at: comment.updated_at().as_unix_seconds(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AttachmentDto {
    pub id: i64,
    pub task_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub created_at: i64,
}

impl From<&todooshka_domain::AttachmentMeta> for AttachmentDto {
    fn from(meta: &todooshka_domain::AttachmentMeta) -> Self {
        Self {
            id: meta.id.as_raw(),
            task_id: meta.task_id.as_raw(),
            filename: meta.filename.as_str().to_owned(),
            content_type: meta.content_type.clone(),
            size: meta.size,
            created_at: meta.created_at.as_unix_seconds(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CommentRequest {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubtaskRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct EditSubtaskRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub done: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EditTaskRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub planning: Option<PlanningRequest>,
}

/// Планирование в PATCH: объект присутствует — набор заменяется целиком.
#[derive(Debug, Deserialize)]
pub struct PlanningRequest {
    #[serde(default)]
    pub due_on: Option<String>,
    pub priority: String,
    #[serde(default)]
    pub estimate_minutes: Option<i64>,
}

impl PlanningRequest {
    pub fn parse(self) -> Result<Planning, todooshka_domain::DomainError> {
        let due_on = self.due_on.as_deref().map(DueDate::parse).transpose()?;
        let priority = Priority::from_code(&self.priority)
            .ok_or_else(|| todooshka_domain::DomainError::InvalidPriority(self.priority.clone()))?;
        let estimate = self
            .estimate_minutes
            .map(EstimateMinutes::parse)
            .transpose()?;

        Ok(Planning::new(due_on, priority, estimate))
    }
}

#[derive(Debug, Deserialize)]
pub struct MoveTaskRequest {
    pub status: String,
    pub target_index: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
}

impl UserDto {
    pub fn from_user(user: &todooshka_domain::User) -> Self {
        Self {
            id: user.id().as_raw(),
            username: user.username().as_str().to_owned(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserDto,
    pub expires_at: i64,
}
