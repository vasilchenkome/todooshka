use std::sync::Arc;

use todooshka_domain::{ProjectId, Tag, TagName, TaskId, TaskTag};

use crate::error::ApplicationError;
use crate::ports::TagStore;

/// Теги задачи: свободные метки с регистронезависимой уникальностью.
pub struct TagService {
    tags: Arc<dyn TagStore>,
}

impl TagService {
    pub fn new(tags: Arc<dyn TagStore>) -> Self {
        Self { tags }
    }

    pub async fn list_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<TaskTag>, ApplicationError> {
        Ok(self.tags.list_by_project(project_id).await?)
    }

    pub async fn list_all(&self) -> Result<Vec<Tag>, ApplicationError> {
        Ok(self.tags.list_all().await?)
    }

    pub async fn replace(
        &self,
        task_id: TaskId,
        names: &[String],
    ) -> Result<Vec<Tag>, ApplicationError> {
        let mut parsed = Vec::with_capacity(names.len());
        let mut seen = Vec::new();

        for raw in names {
            let name = TagName::parse(raw)?;
            let key = name.as_str().to_lowercase();
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            parsed.push(name);
        }

        self.tags
            .replace(task_id, &parsed)
            .await?
            .ok_or(ApplicationError::TaskNotFound(task_id))
    }
}
