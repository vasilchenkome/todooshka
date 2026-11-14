//! Shared integration-test harness: an app on a temporary SQLite with an active session.
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
    AgendaService, AttachmentService, AuthService, Clock, CommentService, CreateUser, Login,
    PasswordHasher, ProjectRepository, ProjectService, RepositoryError, SessionStore,
    SessionTokens, SubtaskService, TagService, TaskService, TaskStore, UserStore,
};
use todooshka_domain::{PasswordHash, Timestamp};
use todooshka_http::{AppState, ReadinessProbe, auth::LoginLimiter, router};
use todooshka_sqlite::{
    SqliteAgenda, SqliteAttachments, SqliteComments, SqliteProjects, SqliteSessions,
    SqliteSubtasks, SqliteTags, SqliteTasks, SqliteUsers, connect, migrate,
};
use tower::ServiceExt;

pub const USERNAME: &str = "demo";
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

/// Fast deterministic hasher: these tests are about HTTP, not about Argon2.
pub struct TestHasher;

impl PasswordHasher for TestHasher {
    fn hash(&self, password: &str) -> Result<PasswordHash, RepositoryError> {
        Ok(PasswordHash::parse(&format!("fake:{password}")).expect("хеш непустой"))
    }

    fn verify(&self, password: &str, hash: &PasswordHash) -> bool {
        hash.as_str() == format!("fake:{password}")
    }
}

/// Readiness in tests: the database is already open and migrated.
pub struct ReadyProbe;

#[async_trait::async_trait]
impl ReadinessProbe for ReadyProbe {
    async fn check(&self) -> bool {
        true
    }
}

/// Deterministic tokens: only the hash reaches the DB.
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

/// An app with a created user and one live session.
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
    let agenda = Arc::new(AgendaService::new(Arc::new(SqliteAgenda::new(
        pool.clone(),
    ))));

    let subtasks = Arc::new(SubtaskService::new(
        Arc::new(SqliteSubtasks::new(pool.clone())),
        Arc::clone(&clock) as Arc<dyn Clock>,
    ));

    let tags = Arc::new(TagService::new(Arc::new(SqliteTags::new(pool.clone()))));
    let comments = Arc::new(CommentService::new(
        Arc::new(SqliteComments::new(pool.clone())),
        Arc::clone(&clock) as Arc<dyn Clock>,
    ));
    let attachments = Arc::new(AttachmentService::new(
        Arc::new(SqliteAttachments::new(pool.clone())),
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
            agenda,
            subtasks,
            tags,
            comments,
            attachments,
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

/// Raw request: needed by tests that look at response headers.
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
