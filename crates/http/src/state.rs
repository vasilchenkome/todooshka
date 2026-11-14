use std::sync::Arc;

use todooshka_app::{AuthService, ProjectService, TaskService};

use crate::auth::LoginLimiter;

/// Всё, что нужно HTTP-слою: сценарии и настройки транспорта.
#[derive(Clone)]
pub struct AppState {
    pub projects: Arc<ProjectService>,
    pub tasks: Arc<TaskService>,
    pub auth: Arc<AuthService>,
    pub login_limiter: LoginLimiter,
    /// Флаг `Secure` у cookie: включается только когда приложение за HTTPS.
    pub cookie_secure: bool,
}
