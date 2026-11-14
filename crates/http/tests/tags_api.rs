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
async fn replace_tags_and_read_links() {
    let app = app().await;
    let (project_id, task_id) = project_and_task(&app).await;

    let (status, tags) = request(
        &app,
        Method::PUT,
        &format!("/api/tasks/{task_id}/tags"),
        Some(json!({ "tags": ["#банк", "Банк", "релиз"] })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{tags}");
    let names: Vec<&str> = tags
        .as_array()
        .expect("массив")
        .iter()
        .map(|tag| tag["name"].as_str().expect("имя"))
        .collect();
    assert_eq!(names, vec!["банк", "релиз"]);

    let (status, links) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/task-tags"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(links.as_array().expect("связи").len(), 2);
    assert_eq!(links[0]["task_id"], task_id);

    let (status, all) = request(&app, Method::GET, "/api/tags", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(all.as_array().expect("теги").len(), 2);

    let (status, cleared) = request(
        &app,
        Method::PUT,
        &format!("/api/tasks/{task_id}/tags"),
        Some(json!({ "tags": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(cleared.as_array().expect("массив").is_empty());
}

#[tokio::test]
async fn invalid_and_missing_are_rejected() {
    let app = app().await;
    let (_, task_id) = project_and_task(&app).await;

    let (status, body) = request(
        &app,
        Method::PUT,
        &format!("/api/tasks/{task_id}/tags"),
        Some(json!({ "tags": ["я".repeat(33)] })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation_failed");

    let (status, _) = request(
        &app,
        Method::PUT,
        "/api/tasks/999/tags",
        Some(json!({ "tags": ["банк"] })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
