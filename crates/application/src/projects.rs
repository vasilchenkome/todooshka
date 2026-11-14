use std::sync::Arc;

use todooshka_domain::{Project, ProjectColor, ProjectDraft, ProjectId, ProjectName};

use crate::error::ApplicationError;
use crate::ports::{Clock, ProjectRepository};

/// Создать проект: имя и цвет проверяются до обращения к хранилищу.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProject {
    pub name: String,
    pub description: String,
    pub color: Option<String>,
}

/// Частичное изменение проекта: заданы только те поля, которые меняются.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateProject {
    pub id: ProjectId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub archived: Option<bool>,
}

/// Сценарии проектов доски. Транзакций здесь нет: каждая операция — один факт хранения.
pub struct ProjectService {
    projects: Arc<dyn ProjectRepository>,
    clock: Arc<dyn Clock>,
}

impl ProjectService {
    pub fn new(projects: Arc<dyn ProjectRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { projects, clock }
    }

    pub async fn create(&self, command: CreateProject) -> Result<Project, ApplicationError> {
        let draft = ProjectDraft::new(
            &command.name,
            &command.description,
            command.color.as_deref(),
        )?;
        let project = self.projects.create(draft, self.clock.now()).await?;
        Ok(project)
    }

    pub async fn list(&self, include_archived: bool) -> Result<Vec<Project>, ApplicationError> {
        let projects = self.projects.list(include_archived).await?;
        Ok(projects)
    }

    /// Применяет изменения одним сохранением.
    ///
    /// Все значения проверяются до первой мутации, поэтому ошибка валидации не оставляет
    /// проект наполовину изменённым. Совпадающие значения не считаются изменением и не
    /// сдвигают `updated_at`.
    pub async fn update(&self, command: UpdateProject) -> Result<Project, ApplicationError> {
        let name = command
            .name
            .as_deref()
            .map(ProjectName::parse)
            .transpose()?;
        let color = command
            .color
            .as_deref()
            .map(ProjectColor::parse)
            .transpose()?;

        let mut project = self.require(command.id).await?;
        let now = self.clock.now();
        let mut changed = false;

        if let Some(name) = name
            && name != *project.name()
        {
            project.rename(name, now);
            changed = true;
        }

        if let Some(description) = command.description.as_deref()
            && description != project.description()
        {
            project.describe(description, now)?;
            changed = true;
        }

        if let Some(color) = color
            && color != *project.color()
        {
            project.recolor(color, now);
            changed = true;
        }

        if let Some(archived) = command.archived
            && archived != project.is_archived()
        {
            if archived {
                project.archive(now);
            } else {
                project.unarchive(now);
            }
            changed = true;
        }

        if changed {
            self.projects.save(&project).await?;
        }

        Ok(project)
    }

    /// Общая проверка существования: одинаковое «не найдено» для всех правок.
    async fn require(&self, id: ProjectId) -> Result<Project, ApplicationError> {
        self.projects
            .get(id)
            .await?
            .ok_or(ApplicationError::ProjectNotFound(id))
    }
}
