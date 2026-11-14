mod support;

use axum::http::{Method, StatusCode, header};
use serde_json::json;

use support::{TestApp, app, read_json, request, request_anon, send};

async fn create_project(app: &TestApp) -> i64 {
    let (status, body) = request(
        app,
        Method::POST,
        "/api/projects",
        Some(json!({ "name": "Экспорт" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_i64().expect("id проекта")
}

#[tokio::test]
async fn export_returns_a_download_without_secrets() {
    let app = app().await;
    let project_id = create_project(&app).await;
    let (_, task) = request(
        &app,
        Method::POST,
        &format!("/api/projects/{project_id}/tasks"),
        Some(json!({ "title": "Задача для экспорта" })),
    )
    .await;
    let task_id = task["id"].as_i64().expect("id задачи");
    request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/subtasks"),
        Some(json!({ "title": "Пункт" })),
    )
    .await;
    request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/comments"),
        Some(json!({ "body": "Комментарий" })),
    )
    .await;

    let response = send(&app, Method::GET, "/api/export", None, true).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/json; charset=utf-8"
    );
    assert!(
        response.headers()[header::CONTENT_DISPOSITION]
            .to_str()
            .expect("строка")
            .starts_with("attachment; filename=\"todooshka-")
    );

    let (_, body) = read_json(response).await;
    assert_eq!(body["format"], "todooshka.export");
    assert_eq!(body["version"], 1);
    assert_eq!(body["projects"][0]["name"], "Экспорт");
    assert_eq!(
        body["projects"][0]["tasks"][0]["title"],
        "Задача для экспорта"
    );
    assert_eq!(
        body["projects"][0]["tasks"][0]["subtasks"][0]["title"],
        "Пункт"
    );
    assert_eq!(
        body["projects"][0]["tasks"][0]["comments"][0]["body"],
        "Комментарий"
    );

    let serialized = body.to_string();
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("session"));
    assert!(!serialized.contains("\"data\""));
}

#[tokio::test]
async fn export_requires_a_session() {
    let app = app().await;
    let (status, _) = request_anon(&app, Method::GET, "/api/export", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
