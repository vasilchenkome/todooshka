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
async fn comment_roundtrip() {
    let app = app().await;
    let (project_id, task_id) = project_and_task(&app).await;

    let (status, created) = request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/comments"),
        Some(json!({ "body": "  Первый комментарий  " })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["body"], "Первый комментарий");
    assert_eq!(created["task_id"], task_id);
    let comment_id = created["id"].as_i64().expect("id комментария");

    let (status, list) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/comments"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("лента").len(), 1);

    let (status, edited) = request(
        &app,
        Method::PATCH,
        &format!("/api/comments/{comment_id}"),
        Some(json!({ "body": "Второй" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{edited}");
    assert_eq!(edited["body"], "Второй");

    let (status, _) = request(
        &app,
        Method::DELETE,
        &format!("/api/comments/{comment_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn empty_comment_is_validation_error() {
    let app = app().await;
    let (_, task_id) = project_and_task(&app).await;

    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/tasks/{task_id}/comments"),
        Some(json!({ "body": "   " })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn missing_entities_are_404() {
    let app = app().await;

    let (status, _) = request(
        &app,
        Method::POST,
        "/api/tasks/999/comments",
        Some(json!({ "body": "Текст" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = request(
        &app,
        Method::PATCH,
        "/api/comments/999",
        Some(json!({ "body": "Текст" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
