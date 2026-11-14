use std::sync::Arc;

use todooshka_app::{ProjectService, TaskService};

/// Всё, что нужно HTTP-слою: сценарии в виде трейт-объектов ещё одного слоя не появляется.
#[derive(Clone)]
pub struct AppState {
    pub projects: Arc<ProjectService>,
    pub tasks: Arc<TaskService>,
}
