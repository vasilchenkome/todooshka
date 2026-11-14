use std::sync::Arc;

use async_trait::async_trait;
use todooshka_app::{
    AgendaService, AttachmentService, AuthService, CommentService, ProjectService, SubtaskService,
    TagService, TaskService,
};

use crate::auth::LoginLimiter;

/// Readiness check: the storage adapter reports whether it can serve requests.
#[async_trait]
pub trait ReadinessProbe: Send + Sync {
    async fn check(&self) -> bool;
}

/// Everything the HTTP layer needs: use cases and transport settings.
#[derive(Clone)]
pub struct AppState {
    pub projects: Arc<ProjectService>,
    pub tasks: Arc<TaskService>,
    pub agenda: Arc<AgendaService>,
    pub subtasks: Arc<SubtaskService>,
    pub tags: Arc<TagService>,
    pub comments: Arc<CommentService>,
    pub attachments: Arc<AttachmentService>,
    pub auth: Arc<AuthService>,
    pub login_limiter: LoginLimiter,
    /// The cookie `Secure` flag: enabled only when the app is behind HTTPS.
    pub cookie_secure: bool,
    pub readiness: Arc<dyn ReadinessProbe>,
}
