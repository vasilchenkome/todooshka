//! HTTP-адаптер: роутер, DTO, middleware, статика SPA.
//!
//! Handlers только переводят запрос в вызов сценария и обратно; бизнес-правил здесь нет.

mod dto;
mod error;
mod health;
mod projects;
mod state;
mod tasks;

pub use state::AppState;

use axum::Router;
use axum::routing::{get, patch, post};

/// Собирает роутер приложения. Composition root передаёт готовые сценарии в состоянии.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/projects", get(projects::list).post(projects::create))
        .route(
            "/api/projects/{id}",
            patch(projects::update).delete(projects::archive),
        )
        .route(
            "/api/projects/{project_id}/tasks",
            get(tasks::list).post(tasks::create),
        )
        .route(
            "/api/tasks/{id}",
            patch(tasks::update).delete(tasks::delete),
        )
        .route("/api/tasks/{id}/move", post(tasks::move_task))
        .with_state(state)
}
