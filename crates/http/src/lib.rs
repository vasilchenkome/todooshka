//! HTTP-адаптер: роутер, DTO, middleware, статика SPA.
//!
//! Handlers только переводят запрос в вызов сценария и обратно; бизнес-правил здесь нет.

mod dto;
mod error;
mod health;
mod projects;
mod state;

pub use state::AppState;

use axum::Router;
use axum::routing::{get, patch};

/// Собирает роутер приложения. Composition root передаёт готовые сценарии в состоянии.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/projects", get(projects::list).post(projects::create))
        .route(
            "/api/projects/{id}",
            patch(projects::update).delete(projects::archive),
        )
        .with_state(state)
}
