use std::sync::Arc;

use todooshka_domain::{Comment, CommentBody, CommentId, ProjectId, TaskId};

use crate::error::ApplicationError;
use crate::ports::{Clock, CommentStore};

/// Лента комментариев задачи: добавить, отредактировать, удалить.
pub struct CommentService {
    comments: Arc<dyn CommentStore>,
    clock: Arc<dyn Clock>,
}

impl CommentService {
    pub fn new(comments: Arc<dyn CommentStore>, clock: Arc<dyn Clock>) -> Self {
        Self { comments, clock }
    }

    pub async fn list(&self, project_id: ProjectId) -> Result<Vec<Comment>, ApplicationError> {
        Ok(self.comments.list_by_project(project_id).await?)
    }

    pub async fn add(&self, task_id: TaskId, body: &str) -> Result<Comment, ApplicationError> {
        let body = CommentBody::parse(body)?;
        self.comments
            .add(task_id, body.as_str(), self.clock.now())
            .await?
            .ok_or(ApplicationError::TaskNotFound(task_id))
    }

    pub async fn edit(&self, id: CommentId, body: &str) -> Result<Comment, ApplicationError> {
        let body = CommentBody::parse(body)?;
        self.comments
            .edit(id, body.as_str(), self.clock.now())
            .await?
            .ok_or(ApplicationError::CommentNotFound(id))
    }

    pub async fn delete(&self, id: CommentId) -> Result<(), ApplicationError> {
        if self.comments.delete(id).await? {
            Ok(())
        } else {
            Err(ApplicationError::CommentNotFound(id))
        }
    }
}
