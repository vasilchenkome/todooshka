mod support;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use support::{TestApp, app, request};

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

#[tokio::test]
async fn project_list_includes_task_counts() {
    let app = app().await;
    let project_id = create_project(&app).await;
    create_task(&app, project_id, "В бэклоге", None).await;
    create_task(&app, project_id, "Готово", Some("done")).await;

    let (status, body) = request(&app, Method::GET, "/api/projects", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body[0]["id"], project_id);
    assert_eq!(body[0]["counts"]["backlog"], 1);
    assert_eq!(body[0]["counts"]["in_progress"], 0);
    assert_eq!(body[0]["counts"]["done"], 1);
}

#[tokio::test]
async fn tasks_require_session() {
    let app = app().await;

    let (status, body) =
        support::request_anon(&app, Method::GET, "/api/projects/1/tasks", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");
}
