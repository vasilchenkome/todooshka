mod support;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

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

fn multipart_body(boundary: &str, filename: &str, content: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n\
             Content-Type: text/plain\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

async fn upload(
    app: &TestApp,
    task_id: i64,
    filename: &str,
    content: &[u8],
) -> axum::response::Response {
    let boundary = "X-BOUNDARY";
    let body = multipart_body(boundary, filename, content);

    app.router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/tasks/{task_id}/attachments"))
                .header(header::COOKIE, &app.cookie)
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .expect("запрос"),
        )
        .await
        .expect("ответ")
}

#[tokio::test]
async fn upload_download_and_delete() {
    let app = app().await;
    let (project_id, task_id) = project_and_task(&app).await;

    let response = upload(&app, task_id, "report.txt", b"hello").await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("тело")
        .to_bytes();
    let meta: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(meta["filename"], "report.txt");
    assert_eq!(meta["content_type"], "text/plain; charset=utf-8");
    assert_eq!(meta["size"], 5);
    let attachment_id = meta["id"].as_i64().expect("id файла");

    let (status, list) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/attachments"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("список").len(), 1);

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/attachments/{attachment_id}"))
                .header(header::COOKIE, &app.cookie)
                .body(Body::empty())
                .expect("запрос"),
        )
        .await
        .expect("ответ");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()[header::CONTENT_DISPOSITION]
            .to_str()
            .expect("заголовок")
            .contains("report.txt")
    );
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
    let data = response
        .into_body()
        .collect()
        .await
        .expect("тело")
        .to_bytes();
    assert_eq!(&data[..], b"hello");

    let (status, _) = request(
        &app,
        Method::DELETE,
        &format!("/api/attachments/{attachment_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = request(
        &app,
        Method::GET,
        &format!("/api/projects/{project_id}/attachments"),
        None,
    )
    .await;
    assert!(list.as_array().expect("список").is_empty());
}

#[tokio::test]
async fn upload_to_missing_task_is_404() {
    let app = app().await;

    let response = upload(&app, 999, "a.txt", b"x").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn empty_file_is_rejected() {
    let app = app().await;
    let (_, task_id) = project_and_task(&app).await;

    let response = upload(&app, task_id, "empty.txt", b"").await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
