use async_trait::async_trait;
use todooshka_domain::{
    PasswordHash, Project, ProjectDraft, ProjectId, Task, TaskDraft, TaskId, TaskStatus, Timestamp,
    User, UserId, Username,
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

/// Сколько задач проекта в каждой колонке: один запрос на все проекты доски.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectTaskCount {
    pub project_id: ProjectId,
    pub status: TaskStatus,
    pub tasks: i64,
}

/// Хранилище задач: чтение доски и короткие операции поверх транзакции.
#[async_trait]
pub trait TaskStore: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<Task>, RepositoryError>;

    async fn counts_by_project(&self) -> Result<Vec<ProjectTaskCount>, RepositoryError>;

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

/// Учётные записи: поиск по логину, создание, смена пароля.
#[async_trait]
pub trait UserStore: Send + Sync {
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, RepositoryError>;

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;

    async fn insert(
        &self,
        username: &Username,
        password_hash: &PasswordHash,
        now: Timestamp,
    ) -> Result<User, RepositoryError>;

    async fn update_password(
        &self,
        username: &Username,
        password_hash: &PasswordHash,
    ) -> Result<bool, RepositoryError>;

    async fn list(&self) -> Result<Vec<User>, RepositoryError>;
}

/// Сессия в хранилище: наружу отдаётся только хеш токена, сам токен живёт в cookie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredSession {
    pub id_hash: String,
    pub user_id: UserId,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub last_seen_at: Timestamp,
}

/// Серверные сессии: создание, поиск по хешу, продление отметки, отзыв.
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: &StoredSession) -> Result<(), RepositoryError>;

    async fn find(&self, id_hash: &str) -> Result<Option<StoredSession>, RepositoryError>;

    async fn touch(&self, id_hash: &str, seen_at: Timestamp) -> Result<(), RepositoryError>;

    async fn delete(&self, id_hash: &str) -> Result<(), RepositoryError>;

    async fn delete_for_user(&self, user_id: UserId) -> Result<(), RepositoryError>;
}

/// Хеширование паролей: конкретный алгоритм — деталь инфраструктуры.
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<PasswordHash, RepositoryError>;

    fn verify(&self, password: &str, hash: &PasswordHash) -> bool;
}

/// Выпуск и хеширование токенов сессий: в БД попадает только хеш.
pub trait SessionTokens: Send + Sync {
    /// Возвращает пару (токен для cookie, хеш для БД).
    fn issue(&self) -> (String, String);

    fn hash(&self, token: &str) -> String;
}
