use async_trait::async_trait;
use todooshka_domain::{
    Project, ProjectDraft, ProjectId, Task, TaskDraft, TaskId, TaskStatus, Timestamp,
};

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

/// Хранилище задач: чтение доски и короткие операции поверх транзакции.
#[async_trait]
pub trait TaskStore: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<Task>, RepositoryError>;

    async fn delete(&self, id: TaskId) -> Result<bool, RepositoryError>;

    /// Открывает Unit of Work: все изменения внутри идут одной транзакцией.
    async fn transaction(&self) -> Result<Box<dyn TaskTransaction>, RepositoryError>;
}

/// Транзакция задач: перенос колонки должен быть атомарным, поэтому чтение и запись
/// выполняются в одном Unit of Work, а коммит остаётся за сценарием.
#[async_trait]
pub trait TaskTransaction: Send + Sync {
    /// Задачи колонки в порядке доски (по позиции, затем по id).
    async fn list_column(
        &mut self,
        project_id: ProjectId,
        status: TaskStatus,
    ) -> Result<Vec<Task>, RepositoryError>;

    async fn get(&mut self, id: TaskId) -> Result<Option<Task>, RepositoryError>;

    /// Ставит задачу в конец колонки: позицию выбирает хранилище.
    async fn create(
        &mut self,
        project_id: ProjectId,
        draft: TaskDraft,
        status: TaskStatus,
        now: Timestamp,
    ) -> Result<Task, RepositoryError>;

    async fn save(&mut self, task: &Task) -> Result<(), RepositoryError>;

    async fn commit(&mut self) -> Result<(), RepositoryError>;
}
