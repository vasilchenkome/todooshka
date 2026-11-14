use std::sync::Arc;

use todooshka_domain::{
    AttachmentFilename, AttachmentId, AttachmentMeta, MAX_ATTACHMENT_BYTES,
    MAX_ATTACHMENTS_PER_TASK, ProjectId, TaskId, content_type_for,
};

use crate::error::ApplicationError;
use crate::ports::{AttachmentStore, Clock};

/// Task files: limits are checked before writing, the type comes from the name.
pub struct AttachmentService {
    attachments: Arc<dyn AttachmentStore>,
    clock: Arc<dyn Clock>,
}

pub struct UploadedFile {
    pub meta: AttachmentMeta,
    pub data: Vec<u8>,
}

impl AttachmentService {
    pub fn new(attachments: Arc<dyn AttachmentStore>, clock: Arc<dyn Clock>) -> Self {
        Self { attachments, clock }
    }

    pub async fn list(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<AttachmentMeta>, ApplicationError> {
        Ok(self.attachments.list_by_project(project_id).await?)
    }

    pub async fn add(
        &self,
        task_id: TaskId,
        filename: &str,
        data: Vec<u8>,
    ) -> Result<AttachmentMeta, ApplicationError> {
        let filename = AttachmentFilename::parse(filename)?;
        if data.is_empty() {
            return Err(ApplicationError::Domain(
                todooshka_domain::DomainError::EmptyAttachment,
            ));
        }
        if data.len() > MAX_ATTACHMENT_BYTES {
            return Err(ApplicationError::Domain(
                todooshka_domain::DomainError::AttachmentTooLarge,
            ));
        }
        if self.attachments.count_for_task(task_id).await? >= MAX_ATTACHMENTS_PER_TASK as i64 {
            return Err(ApplicationError::AttachmentsLimitReached);
        }

        let content_type = content_type_for(filename.as_str());
        self.attachments
            .add(
                task_id,
                filename.as_str(),
                content_type,
                &data,
                self.clock.now(),
            )
            .await?
            .ok_or(ApplicationError::TaskNotFound(task_id))
    }

    pub async fn get(&self, id: AttachmentId) -> Result<UploadedFile, ApplicationError> {
        self.attachments
            .get(id)
            .await?
            .map(|(meta, data)| UploadedFile { meta, data })
            .ok_or(ApplicationError::AttachmentNotFound(id))
    }

    pub async fn delete(&self, id: AttachmentId) -> Result<(), ApplicationError> {
        if self.attachments.delete(id).await? {
            Ok(())
        } else {
            Err(ApplicationError::AttachmentNotFound(id))
        }
    }
}
