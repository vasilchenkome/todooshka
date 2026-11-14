//! HTTP-адаптер: роутер, DTO, middleware, статика SPA.
//!
//! Handlers только переводят запрос в вызов сценария и обратно; бизнес-правил здесь нет.

mod health;

use axum::{Router, routing::get};

/// Собирает роутер приложения. Вызывающий код (composition root) отвечает за serving.
pub fn router() -> Router {
    Router::new().route("/api/health", get(health::health))
}
