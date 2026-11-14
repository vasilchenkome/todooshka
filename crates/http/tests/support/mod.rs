//! Общая обвязка интеграционных тестов: приложение на временной SQLite с активной сессией.
#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use http_body_util::BodyExt;
use serde_json::Value;
use tempfile::TempDir;
use todooshka_app::{
    AuthService, Clock, CreateUser, Login, PasswordHasher, ProjectRepository, ProjectService,
    RepositoryError, SessionStore, SessionTokens, TaskService, TaskStore, UserStore,
};
use todooshka_domain::{PasswordHash, Timestamp};
use todooshka_http::{AppState, ReadinessProbe, auth::LoginLimiter, router};
use todooshka_sqlite::{
    SqliteProjects, SqliteSessions, SqliteTasks, SqliteUsers, connect, migrate,
};
use tower::ServiceExt;

pub const USERNAME: &str = "kirill";
pub const PASSWORD: &str = "test-password-123";

pub struct FixedClock(AtomicI64);

impl FixedClock {
    pub fn set(&self, seconds: i64) {
        self.0.store(seconds, Ordering::SeqCst);
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_unix_seconds(self.0.load(Ordering::SeqCst))
    }
}

/// Быстрый детерминированный хешер: тесты про HTTP, а не про Argon2.
pub struct TestHasher;

impl PasswordHasher for TestHasher {
    fn hash(&self, password: &str) -> Result<PasswordHash, RepositoryError> {
        Ok(PasswordHash::parse(&format!("fake:{password}")).expect("хеш непустой"))
    }

    fn verify(&self, password: &str, hash: &PasswordHash) -> bool {
        hash.as_str() == format!("fake:{password}")
    }
}

/// Готовность в тестах: база уже открыта и мигрирована.
pub struct ReadyProbe;

#[async_trait::async_trait]
impl ReadinessProbe for ReadyProbe {
    async fn check(&self) -> bool {
        true
    }
}

/// Детерминированные токены: в БД уходит только хеш.
pub struct TestTokens(AtomicI64);

impl SessionTokens for TestTokens {
    fn issue(&self) -> (String, String) {
        let number = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        let token = format!("token-{number}");
        let hash = self.hash(&token);
        (token, hash)
    }

    fn hash(&self, token: &str) -> String {
        format!("hash-of-{token}")
    }
}

pub struct TestApp {
    pub router: Router,
    pub cookie: String,
    pub auth: Arc<AuthService>,
    pub clock: Arc<FixedClock>,
    _dir: TempDir,
}

/// Приложение с созданным пользователем и одной живой сессией.
pub async fn app() -> TestApp {
    let dir = tempfile::tempdir().expect("временная папка");
    let path = dir.path().join("todooshka.db");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool = connect(&url).await.expect("база открывается");
    migrate(&pool).await.expect("миграции применяются");

    let projects_repository: Arc<dyn ProjectRepository> =
        Arc::new(SqliteProjects::new(pool.clone()));
    let tasks_repository: Arc<dyn TaskStore> = Arc::new(SqliteTasks::new(pool.clone()));
    let clock = Arc::new(FixedClock(AtomicI64::new(1_000)));

    let projects = Arc::new(ProjectService::new(
        Arc::clone(&projects_repository),
        Arc::clone(&clock) as Arc<dyn Clock>,
    ));
    let tasks = Arc::new(TaskService::new(
        tasks_repository,
        projects_repository,
        Arc::clone(&clock) as Arc<dyn Clock>,
    ));

    let auth = Arc::new(AuthService::new(
        Arc::new(SqliteUsers::new(pool.clone())) as Arc<dyn UserStore>,
        Arc::new(SqliteSessions::new(pool.clone())) as Arc<dyn SessionStore>,
        Arc::new(TestHasher) as Arc<dyn PasswordHasher>,
        Arc::new(TestTokens(AtomicI64::new(0))) as Arc<dyn SessionTokens>,
        Arc::clone(&clock) as Arc<dyn Clock>,
    ));
    auth.create_user(CreateUser {
        username: USERNAME.to_owned(),
        password: PASSWORD.to_owned(),
    })
    .await
    .expect("пользователь создан");

    let issued = auth
        .login(Login {
            username: USERNAME.to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await
        .expect("вход выполнен");

    TestApp {
        router: router(AppState {
            projects,
            tasks,
            auth: Arc::clone(&auth),
            login_limiter: LoginLimiter::default(),
            cookie_secure: false,
            readiness: Arc::new(ReadyProbe),
        }),
        cookie: format!("todooshka_session={}", issued.token),
        auth,
        clock,
        _dir: dir,
    }
}

/// Сырой запрос: нужен тестам, которые смотрят на заголовки ответа.
pub async fn send(
    app: &TestApp,
    method: Method,
    uri: &str,
    payload: Option<Value>,
    with_cookie: bool,
) -> Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if with_cookie {
        builder = builder.header(header::COOKIE, &app.cookie);
    }

    let body = match payload {
        Some(value) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };

    app.router
        .clone()
        .oneshot(builder.body(body).expect("запрос собирается"))
        .await
        .expect("роутер отвечает")
}

pub async fn read_json(response: Response) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("тело читается")
        .to_bytes();

    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("ответ — JSON")
    };

    (status, json)
}

pub async fn request(
    app: &TestApp,
    method: Method,
    uri: &str,
    payload: Option<Value>,
) -> (StatusCode, Value) {
    read_json(send(app, method, uri, payload, true).await).await
}

pub async fn request_anon(
    app: &TestApp,
    method: Method,
    uri: &str,
    payload: Option<Value>,
) -> (StatusCode, Value) {
    read_json(send(app, method, uri, payload, false).await).await
}
