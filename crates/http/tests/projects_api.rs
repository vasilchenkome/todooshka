use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use todooshka_app::{Clock, ProjectRepository, ProjectService};
use todooshka_domain::Timestamp;
use todooshka_http::{AppState, router};
use todooshka_sqlite::{SqliteProjects, connect, migrate};
use tower::ServiceExt;

struct FixedClock(AtomicI64);

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_unix_seconds(self.0.load(Ordering::SeqCst))
    }
}

struct TestApp {
    router: Router,
    _dir: TempDir,
}

async fn app() -> TestApp {
    let dir = tempfile::tempdir().expect("временная папка");
    let path = dir.path().join("todooshka.db");
    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
    let pool = connect(&url).await.expect("база открывается");
    migrate(&pool).await.expect("миграции применяются");

    let repository: Arc<dyn ProjectRepository> = Arc::new(SqliteProjects::new(pool));
    let service = ProjectService::new(repository, Arc::new(FixedClock(AtomicI64::new(1_000))));

    TestApp {
        router: router(AppState {
            projects: Arc::new(service),
        }),
        _dir: dir,
    }
}

async fn request(
    app: &TestApp,
    method: Method,
    uri: &str,
    payload: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = match payload {
        Some(value) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };

    let response = app
        .router
        .clone()
        .oneshot(builder.body(body).expect("запрос собирается"))
        .await
        .expect("роутер отвечает");

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

async fn create_project(app: &TestApp, name: &str) -> i64 {
    let (status, body) = request(
        app,
        Method::POST,
        "/api/projects",
        Some(json!({ "name": name })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "проект должен создаться");
    body["id"].as_i64().expect("id в ответе")
}

#[tokio::test]
async fn health_stays_liveness_only() {
    let app = app().await;

    let (status, body) = request(&app, Method::GET, "/api/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn create_returns_created_project() {
    let app = app().await;

    let (status, body) = request(
        &app,
        Method::POST,
        "/api/projects",
        Some(json!({ "name": "  Релиз 2.0  ", "description": "подготовка" })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Релиз 2.0");
    assert_eq!(body["description"], "подготовка");
    assert_eq!(body["position"], 0);
    assert_eq!(body["archived"], false);
    assert_eq!(body["created_at"], 1_000);
}

#[tokio::test]
async fn blank_name_returns_validation_error() {
    let app = app().await;

    let (status, body) = request(
        &app,
        Method::POST,
        "/api/projects",
        Some(json!({ "name": "   " })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("текст ошибки")
            .contains("пустым"),
        "сообщение должно объяснять причину"
    );
}

#[tokio::test]
async fn list_hides_archived_until_query_asks() {
    let app = app().await;
    create_project(&app, "Активный").await;
    let archived = create_project(&app, "В архив").await;

    let (status, _) = request(
        &app,
        Method::DELETE,
        &format!("/api/projects/{archived}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, active) = request(&app, Method::GET, "/api/projects", None).await;
    let (_, all) = request(
        &app,
        Method::GET,
        "/api/projects?include_archived=true",
        None,
    )
    .await;

    assert_eq!(active.as_array().expect("массив").len(), 1);
    assert_eq!(all.as_array().expect("массив").len(), 2);
    assert_eq!(all[1]["archived"], true);
}

#[tokio::test]
async fn patch_renames_project() {
    let app = app().await;
    let id = create_project(&app, "Старое").await;

    let (status, body) = request(
        &app,
        Method::PATCH,
        &format!("/api/projects/{id}"),
        Some(json!({ "name": "Новое" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Новое");
    assert_eq!(body["updated_at"], 1_000);
}

#[tokio::test]
async fn invalid_patch_changes_nothing() {
    let app = app().await;
    let id = create_project(&app, "Старое").await;

    let (status, body) = request(
        &app,
        Method::PATCH,
        &format!("/api/projects/{id}"),
        Some(json!({ "name": "Новое", "color": "red" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");

    let (_, projects) = request(&app, Method::GET, "/api/projects", None).await;
    assert_eq!(
        projects[0]["name"], "Старое",
        "отвергнутая правка не должна применяться частично"
    );
}

#[tokio::test]
async fn patch_can_archive_and_unarchive() {
    let app = app().await;
    let id = create_project(&app, "Проект").await;

    let (status, body) = request(
        &app,
        Method::PATCH,
        &format!("/api/projects/{id}"),
        Some(json!({ "archived": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["archived"], true);

    let (status, body) = request(
        &app,
        Method::PATCH,
        &format!("/api/projects/{id}"),
        Some(json!({ "archived": false })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["archived"], false);
}

#[tokio::test]
async fn unknown_project_returns_not_found() {
    let app = app().await;

    let (patch_status, patch_body) = request(
        &app,
        Method::PATCH,
        "/api/projects/404",
        Some(json!({ "name": "Новое" })),
    )
    .await;
    let (delete_status, _) = request(&app, Method::DELETE, "/api/projects/404", None).await;

    assert_eq!(patch_status, StatusCode::NOT_FOUND);
    assert_eq!(patch_body["error"]["code"], "not_found");
    assert_eq!(delete_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_archives_instead_of_removing() {
    let app = app().await;
    let id = create_project(&app, "Проект").await;

    let (status, _) = request(&app, Method::DELETE, &format!("/api/projects/{id}"), None).await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, all) = request(
        &app,
        Method::GET,
        "/api/projects?include_archived=true",
        None,
    )
    .await;
    assert_eq!(all[0]["id"], id);
    assert_eq!(all[0]["archived"], true);
}
