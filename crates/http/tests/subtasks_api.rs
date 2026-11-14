mod support;

use axum::http::{Method, StatusCode};
use serde_json::json;

use support::{TestApp, app, request};

async fn project_and_task(app: &TestApp) -> (i64, i64) {
    let (_, project) = request(
        app,
        Method::POST,
        "/api/projects",
        Some(json!({ "name": "Проект" })),
    )
    .await;
    let project_id = project["id"].as_i64().expect("id проекта");

    let (_, task) = request(
        app,
        Method::POST,
        &format!("/api/projects/{project_id}/tasks"),
        Some(json!({ "title": "Задача" })),
    )
    .await;

    (project_id, task["id"].as_i64().expect("id задачи"))
}

#[tokio::test]
async fn checklist_roundtrip() {
    let app = app().await;
    let (project_id, task_id) = project_and_task(&app).await;

    let (status, created) = request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/subtasks"),
        Some(json!({ "title": "  Проверить ретраи  " })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["title"], "Проверить ретраи");
    assert_eq!(created["task_id"], task_id);
    assert_eq!(created["done"], false);
    let subtask_id = created["id"].as_i64().expect("id пункта");

    let (status, list) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/subtasks"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("массив").len(), 1);

    let (status, toggled) = request(
        &app,
        Method::PATCH,
        &format!("/api/subtasks/{subtask_id}"),
        Some(json!({ "done": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{toggled}");
    assert_eq!(toggled["done"], true);

    let (status, _) = request(
        &app,
        Method::DELETE,
        &format!("/api/subtasks/{subtask_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/subtasks"),
        None,
    )
    .await;
    assert!(list.as_array().expect("массив").is_empty());
}

#[tokio::test]
async fn empty_title_is_validation_error() {
    let app = app().await;
    let (_, task_id) = project_and_task(&app).await;

    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/subtasks"),
        Some(json!({ "title": "   " })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn missing_task_and_subtask_are_404() {
    let app = app().await;

    let (status, _) = request(
        &app,
        Method::POST,
        "/api/tasks/999/subtasks",
        Some(json!({ "title": "Пункт" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = request(
        &app,
        Method::PATCH,
        "/api/subtasks/999",
        Some(json!({ "done": true })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
