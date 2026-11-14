//! HTTP-адаптер: роутер, DTO, middleware, статика SPA.
//!
//! Handlers только переводят запрос в вызов сценария и обратно; бизнес-правил здесь нет.

pub mod assets;
pub mod auth;
mod dto;
mod error;
mod health;
mod projects;
mod state;
mod tasks;

pub use state::{AppState, ReadinessProbe};

use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, patch, post};
use tower_http::timeout::TimeoutLayer;

use crate::error::ApiError;

/// Верхняя граница на запрос: зависший клиент не держит соединение бесконечно.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Собирает роутер приложения.
///
/// Публично доступны только health, вход и статика; остальное закрыто сессией, которую
/// проверяет middleware и кладёт пользователя в расширения запроса.
pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/projects", get(projects::list).post(projects::create))
        .route(
            "/projects/{id}",
            patch(projects::update).delete(projects::archive),
        )
        .route(
            "/projects/{project_id}/tasks",
            get(tasks::list).post(tasks::create),
        )
        .route("/tasks/{id}", patch(tasks::update).delete(tasks::delete))
        .route("/tasks/{id}/move", post(tasks::move_task))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ));

    let api = Router::new()
        .route("/health", get(health::health))
        .route("/health/ready", get(health::ready))
        .route("/auth/login", post(auth::login))
        .merge(protected)
        .fallback(api_not_found)
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .fallback(assets::serve)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
}

async fn api_not_found() -> impl IntoResponse {
    ApiError::not_found("маршрут не найден")
}
