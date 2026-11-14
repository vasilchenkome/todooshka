use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use todooshka_app::{ApplicationError, AuthenticatedUser, Login, SESSION_TTL_SECONDS};

use crate::dto::{LoginRequest, LoginResponse, UserDto};
use crate::error::ApiError;
use crate::state::AppState;

/// Имя cookie сессии. Одно место, где оно вообще упоминается.
pub const SESSION_COOKIE: &str = "todooshka_session";

/// Не больше 5 неудачных попыток в минуту на логин: без IP и внешних зависимостей.
#[derive(Clone, Default)]
pub struct LoginLimiter {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl LoginLimiter {
    const LIMIT: usize = 5;
    const WINDOW: Duration = Duration::from_secs(60);

    pub fn is_blocked(&self, username: &str) -> bool {
        self.recent(username).len() >= Self::LIMIT
    }

    pub fn record_failure(&self, username: &str) {
        let mut attempts = self.attempts.lock().expect("мьютекс не отравлен");
        attempts
            .entry(normalize(username))
            .or_default()
            .push(Instant::now());
    }

    pub fn clear(&self, username: &str) {
        self.attempts
            .lock()
            .expect("мьютекс не отравлен")
            .remove(&normalize(username));
    }

    fn recent(&self, username: &str) -> Vec<Instant> {
        let mut attempts = self.attempts.lock().expect("мьютекс не отравлен");
        let entry = attempts.entry(normalize(username)).or_default();
        entry.retain(|attempt| attempt.elapsed() < Self::WINDOW);
        entry.clone()
    }
}

fn normalize(username: &str) -> String {
    username.trim().to_lowercase()
}

/// Достаёт токен из заголовка `Cookie`; других cookie приложению не нужно.
pub fn session_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            (name.trim() == SESSION_COOKIE).then(|| value.trim().to_owned())
        })
        .find(|value| !value.is_empty())
}

/// Middleware: всё, что ниже, требует валидную сессию.
pub async fn require_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = session_token(request.headers()).unwrap_or_default();
    let Some(user) = state.auth.authenticate(&token).await? else {
        return Err(ApiError::unauthorized("требуется вход"));
    };

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    if state.login_limiter.is_blocked(&request.username) {
        return Err(ApiError::too_many_attempts());
    }

    let issued = match state
        .auth
        .login(Login {
            username: request.username.clone(),
            password: request.password,
        })
        .await
    {
        Ok(issued) => issued,
        Err(ApplicationError::InvalidCredentials) => {
            state.login_limiter.record_failure(&request.username);
            return Err(ApplicationError::InvalidCredentials.into());
        }
        Err(error) => return Err(error.into()),
    };

    state.login_limiter.clear(&request.username);

    let cookie = session_cookie(&issued.token, SESSION_TTL_SECONDS, state.cookie_secure);
    let body = Json(LoginResponse {
        user: UserDto::from_user(&issued.user),
        expires_at: issued.expires_at.as_unix_seconds(),
    });

    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie)], body).into_response())
}

pub async fn logout(State(state): State<AppState>, request: Request) -> Result<Response, ApiError> {
    if let Some(token) = session_token(request.headers()) {
        state.auth.logout(&token).await?;
    }

    let cookie = session_cookie("", 0, state.cookie_secure);
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, cookie)]).into_response())
}

pub async fn me(user: axum::Extension<AuthenticatedUser>) -> Json<UserDto> {
    Json(UserDto {
        id: user.id.as_raw(),
        username: user.username.clone(),
    })
}

fn session_cookie(token: &str, max_age: i64, secure: bool) -> String {
    let secure = if secure { "; Secure" } else { "" };
    format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}")
}
