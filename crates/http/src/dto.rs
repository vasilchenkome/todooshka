use serde::{Deserialize, Serialize};
use todooshka_domain::Project;

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
