use async_trait::async_trait;
use todooshka_domain::{
    AttachmentId, AttachmentMeta, Comment, CommentId, DueDate, PasswordHash, Project, ProjectDraft,
    ProjectId, Subtask, SubtaskId, Tag, TagName, Task, TaskDraft, TaskId, TaskStatus, TaskTag,
    Timestamp, User, UserId, Username,
};

use crate::agenda::AgendaEntry;
use crate::error::RepositoryError;
use crate::export::ExportData;

/// Project storage in the language of the domain.
///
/// Contract: `list` returns active projects (or all, with archived ones) in board order,
/// and `create` picks the end position itself — a storage detail, not a use-case one.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn list(&self, include_archived: bool) -> Result<Vec<Project>, RepositoryError>;

    async fn get(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError>;

    async fn create(&self, draft: ProjectDraft, now: Timestamp)
    -> Result<Project, RepositoryError>;

    async fn save(&self, project: &Project) -> Result<(), RepositoryError>;
}

/// Read side of the Today screen: open tasks whose due date is on or before `through`.
#[async_trait]
pub trait AgendaReader: Send + Sync {
    async fn list_open_due_until(
        &self,
        through: &DueDate,
    ) -> Result<Vec<AgendaEntry>, RepositoryError>;
}

/// Full plain snapshot needed by the export; no auth tables are visible here.
#[async_trait]
pub trait ExportReader: Send + Sync {
    async fn read(&self) -> Result<ExportData, RepositoryError>;
}

/// Time source: use cases never call the system clock, so tests stay deterministic.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

/// How many of a project's tasks sit in each column: one query for every board project.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectTaskCount {
    pub project_id: ProjectId,
    pub status: TaskStatus,
    pub tasks: i64,
}

/// Task storage: board reads and short operations on top of a transaction.
#[async_trait]
pub trait TaskStore: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<Task>, RepositoryError>;

    async fn counts_by_project(&self) -> Result<Vec<ProjectTaskCount>, RepositoryError>;

    async fn delete(&self, id: TaskId) -> Result<bool, RepositoryError>;

    /// Opens a Unit of Work: every change inside goes through a single transaction.
    async fn transaction(&self) -> Result<Box<dyn TaskTransaction>, RepositoryError>;
}

/// Task transaction: a column move must be atomic, so reads and writes share
/// one Unit of Work while the commit stays with the use case.
#[async_trait]
pub trait TaskTransaction: Send + Sync {
    /// The column's tasks in board order (by position, then by id).
    async fn list_column(
        &mut self,
        project_id: ProjectId,
        status: TaskStatus,
    ) -> Result<Vec<Task>, RepositoryError>;

    async fn get(&mut self, id: TaskId) -> Result<Option<Task>, RepositoryError>;

    /// Appends the task to the end of the column: the storage picks the position.
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

/// Task checklist: items are read in bulk per project, operations are targeted.
#[async_trait]
pub trait SubtaskStore: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId)
    -> Result<Vec<Subtask>, RepositoryError>;

    /// `None` means there is no such task: the use case decides which error to return.
    async fn add(
        &self,
        task_id: TaskId,
        title: &str,
        now: Timestamp,
    ) -> Result<Option<Subtask>, RepositoryError>;

    async fn update(
        &self,
        id: SubtaskId,
        title: Option<&str>,
        done: Option<bool>,
        now: Timestamp,
    ) -> Result<Option<Subtask>, RepositoryError>;

    async fn delete(&self, id: SubtaskId) -> Result<bool, RepositoryError>;
}

/// Tags: project links for the board, the full list for suggestions, set swap per task.
#[async_trait]
pub trait TagStore: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId)
    -> Result<Vec<TaskTag>, RepositoryError>;

    async fn list_all(&self) -> Result<Vec<Tag>, RepositoryError>;

    /// `None` means there is no such task; an empty list is valid and means "tags cleared".
    async fn replace(
        &self,
        task_id: TaskId,
        names: &[TagName],
    ) -> Result<Option<Vec<Tag>>, RepositoryError>;
}

/// Comments: the project feed for counters plus targeted operations.
#[async_trait]
pub trait CommentStore: Send + Sync {
    async fn list_by_project(&self, project_id: ProjectId)
    -> Result<Vec<Comment>, RepositoryError>;

    async fn add(
        &self,
        task_id: TaskId,
        body: &str,
        now: Timestamp,
    ) -> Result<Option<Comment>, RepositoryError>;

    async fn edit(
        &self,
        id: CommentId,
        body: &str,
        now: Timestamp,
    ) -> Result<Option<Comment>, RepositoryError>;

    async fn delete(&self, id: CommentId) -> Result<bool, RepositoryError>;
}

/// Attachments: metadata per project, bytes only when one file is requested.
#[async_trait]
pub trait AttachmentStore: Send + Sync {
    async fn list_by_project(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<AttachmentMeta>, RepositoryError>;

    async fn count_for_task(&self, task_id: TaskId) -> Result<i64, RepositoryError>;

    async fn add(
        &self,
        task_id: TaskId,
        filename: &str,
        content_type: &str,
        data: &[u8],
        now: Timestamp,
    ) -> Result<Option<AttachmentMeta>, RepositoryError>;

    async fn get(
        &self,
        id: AttachmentId,
    ) -> Result<Option<(AttachmentMeta, Vec<u8>)>, RepositoryError>;

    async fn delete(&self, id: AttachmentId) -> Result<bool, RepositoryError>;
}

/// Accounts: lookup by login, creation, password change.
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

/// A stored session: only the token hash goes out, the token itself lives in a cookie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredSession {
    pub id_hash: String,
    pub user_id: UserId,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub last_seen_at: Timestamp,
}

/// Server sessions: create, look up by hash, refresh the timestamp, revoke.
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: &StoredSession) -> Result<(), RepositoryError>;

    async fn find(&self, id_hash: &str) -> Result<Option<StoredSession>, RepositoryError>;

    async fn touch(&self, id_hash: &str, seen_at: Timestamp) -> Result<(), RepositoryError>;

    async fn delete(&self, id_hash: &str) -> Result<(), RepositoryError>;

    async fn delete_for_user(&self, user_id: UserId) -> Result<(), RepositoryError>;
}

/// Password hashing: the concrete algorithm is an infrastructure detail.
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<PasswordHash, RepositoryError>;

    fn verify(&self, password: &str, hash: &PasswordHash) -> bool;
}

/// Issuing and hashing session tokens: only the hash reaches the DB.
pub trait SessionTokens: Send + Sync {
    /// Returns a pair (token for the cookie, hash for the DB).
    fn issue(&self) -> (String, String);

    fn hash(&self, token: &str) -> String;
}
