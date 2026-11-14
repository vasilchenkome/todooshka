mod support;

use axum::http::{Method, StatusCode, header};
use serde_json::json;

use support::{TestApp, app, request, request_anon, send};

async fn create_project(app: &TestApp) -> i64 {
    let (status, body) = request(
        app,
        Method::POST,
        "/api/projects",
        Some(json!({ "name": "Прод-проверка" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_i64().expect("id проекта")
}

async fn create_task(app: &TestApp, project_id: i64, title: &str, status: &str) -> i64 {
    let (response, body) = request(
        app,
        Method::POST,
        &format!("/api/projects/{project_id}/tasks"),
        Some(json!({ "title": title, "status": status })),
    )
    .await;
    assert_eq!(response, StatusCode::CREATED, "{body}");
    body["id"].as_i64().expect("id задачи")
}

async fn set_due(app: &TestApp, task_id: i64, due_on: Option<&str>) {
    let (status, body) = request(
        app,
        Method::PATCH,
        &format!("/api/tasks/{task_id}"),
        Some(json!({
            "planning": {
                "due_on": due_on,
                "priority": "high"
            }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn agenda_buckets_boundaries_and_excludes_done() {
    let app = app().await;
    let project_id = create_project(&app).await;

    let overdue = create_task(&app, project_id, "Просрочено", "backlog").await;
    set_due(&app, overdue, Some("2026-11-13")).await;
    let today = create_task(&app, project_id, "Сегодня", "in_progress").await;
    set_due(&app, today, Some("2026-11-14")).await;
    let tomorrow = create_task(&app, project_id, "Завтра", "backlog").await;
    set_due(&app, tomorrow, Some("2026-11-15")).await;
    let week_end = create_task(&app, project_id, "Через семь", "backlog").await;
    set_due(&app, week_end, Some("2026-11-21")).await;
    let later = create_task(&app, project_id, "Через восемь", "backlog").await;
    set_due(&app, later, Some("2026-11-22")).await;
    let done = create_task(&app, project_id, "Готово", "done").await;
    set_due(&app, done, Some("2026-11-14")).await;
    let no_due = create_task(&app, project_id, "Без срока", "backlog").await;
    set_due(&app, no_due, None).await;

    let (status, body) = request(&app, Method::GET, "/api/agenda?today=2026-11-14", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["date"], "2026-11-14");
    let titles = |section: &str| -> Vec<&str> {
        body[section]
            .as_array()
            .expect("массив")
            .iter()
            .map(|entry| entry["title"].as_str().expect("title"))
            .collect()
    };
    assert_eq!(titles("overdue"), ["Просрочено"]);
    assert_eq!(titles("today"), ["Сегодня"]);
    assert_eq!(titles("week"), ["Завтра", "Через семь"]);
    assert_eq!(body["today"][0]["project_name"], "Прод-проверка");
}

#[tokio::test]
async fn agenda_rejects_missing_and_invalid_today() {
    let app = app().await;

    let (missing, missing_body) = request(&app, Method::GET, "/api/agenda", None).await;
    assert_eq!(missing, StatusCode::BAD_REQUEST);
    assert_eq!(missing_body["error"]["code"], "missing_today");

    let (invalid, invalid_body) =
        request(&app, Method::GET, "/api/agenda?today=2026-02-30", None).await;
    assert_eq!(invalid, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_body["error"]["code"], "invalid_today");
}

#[tokio::test]
async fn agenda_is_not_cached_and_requires_session() {
    let app = app().await;

    let response = send(
        &app,
        Method::GET,
        "/api/agenda?today=2026-11-14",
        None,
        true,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

    let (status, _) = request_anon(&app, Method::GET, "/api/agenda?today=2026-11-14", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
