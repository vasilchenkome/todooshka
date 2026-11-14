use async_trait::async_trait;
use todooshka_domain::{Project, ProjectDraft, ProjectId, Timestamp};

use crate::error::RepositoryError;

/// Хранилище проектов на языке домена.
///
/// Контракт: `list` возвращает активные проекты (или все с архивными) в порядке доски,
/// а `create` сам выбирает позицию в конце списка — это деталь хранения, не сценария.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn list(&self, include_archived: bool) -> Result<Vec<Project>, RepositoryError>;

    async fn get(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError>;

    async fn create(&self, draft: ProjectDraft, now: Timestamp)
    -> Result<Project, RepositoryError>;

    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
}

/// Источник времени: сценарии не вызывают системные часы напрямую, тесты — детерминированы.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}
