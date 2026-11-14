use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::AppState;

/// Liveness response: the process is alive. The DB check lives under `/api/health/ready`.
#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        name: "todooshka",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Readiness response: the database answers, so requests can be accepted.
#[derive(Debug, Serialize)]
pub struct Ready {
    pub status: &'static str,
}

pub async fn ready(State(state): State<AppState>) -> Response {
    let status = if state.readiness.check().await {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "unavailable")
    };

    (status.0, Json(Ready { status: status.1 })).into_response()
}
