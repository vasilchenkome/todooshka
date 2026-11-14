use std::sync::Arc;

use todooshka_domain::{ProjectId, Subtask, SubtaskId, SubtaskTitle, TaskId};

use crate::error::ApplicationError;
use crate::ports::{Clock, SubtaskStore};

/// Сценарии чек-листа: добавить, переименовать, отметить, удалить.
pub struct SubtaskService {
    subtasks: Arc<dyn SubtaskStore>,
    clock: Arc<dyn Clock>,
}

impl SubtaskService {
    pub fn new(subtasks: Arc<dyn SubtaskStore>, clock: Arc<dyn Clock>) -> Self {
        Self { subtasks, clock }
    }

    pub async fn list(&self, project_id: ProjectId) -> Result<Vec<Subtask>, ApplicationError> {
        Ok(self.subtasks.list_by_project(project_id).await?)
    }

    pub async fn add(&self, task_id: TaskId, title: &str) -> Result<Subtask, ApplicationError> {
        let title = SubtaskTitle::parse(title)?;
        self.subtasks
            .add(task_id, title.as_str(), self.clock.now())
            .await?
            .ok_or(ApplicationError::TaskNotFound(task_id))
    }

    pub async fn edit(
        &self,
        id: SubtaskId,
        title: Option<&str>,
        done: Option<bool>,
    ) -> Result<Subtask, ApplicationError> {
        let title = title
            .map(SubtaskTitle::parse)
            .transpose()?
            .map(|value| value.as_str().to_owned());

        self.subtasks
            .update(id, title.as_deref(), done, self.clock.now())
            .await?
            .ok_or(ApplicationError::SubtaskNotFound(id))
    }

    pub async fn delete(&self, id: SubtaskId) -> Result<(), ApplicationError> {
        if self.subtasks.delete(id).await? {
            Ok(())
        } else {
            Err(ApplicationError::SubtaskNotFound(id))
        }
    }
}
