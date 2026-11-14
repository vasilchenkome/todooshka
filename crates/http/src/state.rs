use std::sync::Arc;

use async_trait::async_trait;
use todooshka_app::{
    AttachmentService, AuthService, CommentService, ProjectService, SubtaskService, TagService,
    TaskService,
};

use crate::auth::LoginLimiter;

/// Проверка готовности: адаптер хранилища отвечает, можно ли обслуживать запросы.
#[async_trait]
pub trait ReadinessProbe: Send + Sync {
    async fn check(&self) -> bool;
}

/// Всё, что нужно HTTP-слою: сценарии и настройки транспорта.
#[derive(Clone)]
pub struct AppState {
    pub projects: Arc<ProjectService>,
    pub tasks: Arc<TaskService>,
    pub subtasks: Arc<SubtaskService>,
    pub tags: Arc<TagService>,
    pub comments: Arc<CommentService>,
    pub attachments: Arc<AttachmentService>,
    pub auth: Arc<AuthService>,
    pub login_limiter: LoginLimiter,
    /// Флаг `Secure` у cookie: включается только когда приложение за HTTPS.
    pub cookie_secure: bool,
    pub readiness: Arc<dyn ReadinessProbe>,
}
