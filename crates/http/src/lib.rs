//! HTTP-адаптер: роутер, DTO, middleware, статика SPA.
//!
//! Handlers только переводят запрос в вызов сценария и обратно; бизнес-правил здесь нет.

pub mod auth;
mod dto;
mod error;
mod health;
mod projects;
mod state;
mod tasks;

pub use state::AppState;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{get, patch, post};

/// Собирает роутер приложения.
///
/// Публично доступны только health и вход; остальное закрыто сессией, которую проверяет
/// middleware и кладёт пользователя в расширения запроса.
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
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
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ));

    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/auth/login", post(auth::login))
        .merge(protected)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(state)
}
