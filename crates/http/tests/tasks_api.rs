use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use todooshka_app::{Clock, ProjectRepository, ProjectService, TaskService, TaskStore};
use todooshka_domain::Timestamp;
use todooshka_http::{AppState, router};
use todooshka_sqlite::{SqliteProjects, SqliteTasks, connect, migrate};
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

    let projects_repository: Arc<dyn ProjectRepository> =
        Arc::new(SqliteProjects::new(pool.clone()));
    let tasks_repository: Arc<dyn TaskStore> = Arc::new(SqliteTasks::new(pool.clone()));
    let clock: Arc<dyn Clock> = Arc::new(FixedClock(AtomicI64::new(1_000)));

    let projects = Arc::new(ProjectService::new(
        Arc::clone(&projects_repository),
        Arc::clone(&clock),
    ));
    let tasks = Arc::new(TaskService::new(
        tasks_repository,
        projects_repository,
        clock,
    ));

    TestApp {
        router: router(AppState { projects, tasks }),
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

async fn create_project(app: &TestApp) -> i64 {
    let (status, body) = request(
        app,
        Method::POST,
        "/api/projects",
        Some(json!({ "name": "Проект" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_i64().expect("id проекта")
}

async fn create_task(app: &TestApp, project_id: i64, title: &str, status: Option<&str>) -> i64 {
    let payload = match status {
        Some(status) => json!({ "title": title, "status": status }),
        None => json!({ "title": title }),
    };
    let (response, body) = request(
        app,
        Method::POST,
        &format!("/api/projects/{project_id}/tasks"),
        Some(payload),
    )
    .await;
    assert_eq!(
        response,
        StatusCode::CREATED,
        "задача должна создаться: {body}"
    );
    body["id"].as_i64().expect("id задачи")
}

fn titles(body: &Value) -> Vec<&str> {
    body.as_array()
        .expect("массив задач")
        .iter()
        .map(|task| task["title"].as_str().expect("заголовок"))
        .collect()
}

#[tokio::test]
async fn create_task_defaults_to_backlog() {
    let app = app().await;
    let project_id = create_project(&app).await;

    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/projects/{project_id}/tasks"),
        Some(json!({ "title": "  Сделать релиз  " })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["title"], "Сделать релиз");
    assert_eq!(body["status"], "backlog");
    assert_eq!(body["position"], 1.0);
    assert_eq!(body["project_id"], project_id);
    assert!(body["completed_at"].is_null());
}

#[tokio::test]
async fn create_done_task_sets_completion_time() {
    let app = app().await;
    let project_id = create_project(&app).await;
    create_task(&app, project_id, "Старая задача", Some("done")).await;

    let (_, body) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/tasks"),
        None,
    )
    .await;

    assert_eq!(body[0]["status"], "done");
    assert_eq!(body[0]["completed_at"], 1_000);
}

#[tokio::test]
async fn unknown_status_is_a_validation_error() {
    let app = app().await;
    let project_id = create_project(&app).await;

    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/projects/{project_id}/tasks"),
        Some(json!({ "title": "Задача", "status": "archived" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn tasks_of_unknown_project_return_not_found() {
    let app = app().await;

    let (status, body) = request(&app, Method::GET, "/api/projects/404/tasks", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn board_lists_tasks_in_column_order() {
    let app = app().await;
    let project_id = create_project(&app).await;
    create_task(&app, project_id, "Готово", Some("done")).await;
    create_task(&app, project_id, "В работе", Some("in_progress")).await;
    create_task(&app, project_id, "Бэклог", None).await;

    let (status, body) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/tasks"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(titles(&body), ["Бэклог", "В работе", "Готово"]);
}

#[tokio::test]
async fn patch_task_updates_text_fields() {
    let app = app().await;
    let project_id = create_project(&app).await;
    let task_id = create_task(&app, project_id, "Старинная", None).await;

    let (status, body) = request(
        &app,
        Method::PATCH,
        &format!("/api/tasks/{task_id}"),
        Some(json!({ "title": "Новая", "description": "подробности" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "Новая");
    assert_eq!(body["description"], "подробности");
}

#[tokio::test]
async fn move_task_changes_column_and_completion() {
    let app = app().await;
    let project_id = create_project(&app).await;
    let task_id = create_task(&app, project_id, "Задача", None).await;

    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/move"),
        Some(json!({ "status": "done", "target_index": 0 })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "done");
    assert_eq!(body["completed_at"], 1_000);
}

#[tokio::test]
async fn move_task_reorders_within_column() {
    let app = app().await;
    let project_id = create_project(&app).await;
    create_task(&app, project_id, "Первая", None).await;
    create_task(&app, project_id, "Вторая", None).await;
    let third = create_task(&app, project_id, "Третья", None).await;

    let (status, _) = request(
        &app,
        Method::POST,
        &format!("/api/tasks/{third}/move"),
        Some(json!({ "status": "backlog", "target_index": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/tasks"),
        None,
    )
    .await;
    assert_eq!(titles(&body), ["Первая", "Третья", "Вторая"]);
}

#[tokio::test]
async fn move_with_unknown_status_is_validation_error() {
    let app = app().await;
    let project_id = create_project(&app).await;
    let task_id = create_task(&app, project_id, "Задача", None).await;

    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/move"),
        Some(json!({ "status": "archived", "target_index": 0 })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn move_unknown_task_returns_not_found() {
    let app = app().await;

    let (status, body) = request(
        &app,
        Method::POST,
        "/api/tasks/404/move",
        Some(json!({ "status": "done", "target_index": 0 })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn delete_task_then_reports_not_found() {
    let app = app().await;
    let project_id = create_project(&app).await;
    let task_id = create_task(&app, project_id, "Задача", None).await;

    let (status, _) = request(&app, Method::DELETE, &format!("/api/tasks/{task_id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(&app, Method::DELETE, &format!("/api/tasks/{task_id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, body) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/tasks"),
        None,
    )
    .await;
    assert!(body.as_array().expect("массив").is_empty());
}
