use std::sync::Arc;

use todooshka_domain::{Project, ProjectColor, ProjectDraft, ProjectId, ProjectName};

use crate::error::ApplicationError;
use crate::ports::{Clock, ProjectRepository};

/// Create a project: name and colour are validated before touching storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProject {
    pub name: String,
    pub description: String,
    pub color: Option<String>,
}

/// A partial project update: only the fields that change are provided.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateProject {
    pub id: ProjectId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub archived: Option<bool>,
}

/// Board project use cases. No transactions here: each operation is one storage fact.
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

    /// Applies the changes with a single save.
    ///
    /// Every value is validated before the first mutation, so a validation error never
    /// leaves the project half-changed. Unchanged values do not count as an edit and
    /// do not bump `updated_at`.
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

    /// Shared existence check: the same "not found" for every edit.
    async fn require(&self, id: ProjectId) -> Result<Project, ApplicationError> {
        self.projects
            .get(id)
            .await?
            .ok_or(ApplicationError::ProjectNotFound(id))
    }
}
