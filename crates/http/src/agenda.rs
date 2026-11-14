use axum::Json;
use axum::extract::{Query, State};
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use todooshka_domain::DueDate;

use crate::dto::AgendaDto;
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AgendaQuery {
    #[serde(default)]
    pub today: Option<String>,
}

/// Read-only agenda; the browser supplies the user's calendar date.
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<AgendaQuery>,
) -> Result<Response, ApiError> {
    let raw = query.today.ok_or_else(|| {
        ApiError::bad_request(
            "missing_today",
            "параметр today обязателен",
            "the today parameter is required",
        )
    })?;
    let today = DueDate::parse(&raw).map_err(|_| {
        ApiError::bad_request(
            "invalid_today",
            "today должен быть реальной датой в формате YYYY-MM-DD",
            "today must be a real date in YYYY-MM-DD format",
        )
    })?;

    let agenda = state.agenda.build(today).await?;
    let response = (
        [(CACHE_CONTROL, "no-store")],
        Json(AgendaDto::from(&agenda)),
    )
        .into_response();

    Ok(response)
}
