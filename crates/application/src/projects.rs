use std::sync::Arc;

use todooshka_domain::{Project, ProjectDraft, ProjectId, ProjectName};

use crate::error::ApplicationError;
use crate::ports::{Clock, ProjectRepository};

/// Создать проект: имя и цвет проверяются до обращения к хранилищу.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProject {
    pub name: String,
    pub description: String,
    pub color: Option<String>,
}

/// Переименовать проект.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenameProject {
    pub id: ProjectId,
    pub name: String,
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

    pub async fn rename(&self, command: RenameProject) -> Result<Project, ApplicationError> {
        let name = ProjectName::parse(&command.name)?;
        let mut project = self.require(command.id).await?;

        project.rename(name, self.clock.now());
        self.projects.save(&project).await?;
        Ok(project)
    }

    pub async fn archive(&self, id: ProjectId) -> Result<Project, ApplicationError> {
        let mut project = self.require(id).await?;
        if project.is_archived() {
            return Ok(project);
        }

        project.archive(self.clock.now());
        self.projects.save(&project).await?;
        Ok(project)
    }

    pub async fn unarchive(&self, id: ProjectId) -> Result<Project, ApplicationError> {
        let mut project = self.require(id).await?;
        if !project.is_archived() {
            return Ok(project);
        }

        project.unarchive(self.clock.now());
        self.projects.save(&project).await?;
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
