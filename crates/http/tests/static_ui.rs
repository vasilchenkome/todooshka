//! SPA assets: one binary serves the shell, the assets and a JSON 404 for unknown API paths.

mod support;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn dist_ready() -> bool {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../frontend/dist/index.html")
        .exists()
}

async fn get(app: &support::TestApp, uri: &str) -> axum::response::Response {
    app.router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("ответ")
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("тело")
        .to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[tokio::test]
async fn serves_index_and_client_routes() {
    if !dist_ready() {
        eprintln!("frontend/dist не собран — проверка статики пропущена");
        return;
    }
    let app = support::app().await;

    for uri in ["/", "/projects/1", "/login"] {
        let response = get(&app, uri).await;
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/html; charset=utf-8",
            "{uri}"
        );
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-cache",
            "{uri}"
        );
        let body = body_text(response).await;
        assert!(body.contains("<div id=\"root\">"), "{uri}: {body}");
    }
}

#[tokio::test]
async fn serves_hashed_asset_with_long_cache() {
    if !dist_ready() {
        eprintln!("frontend/dist не собран — проверка ассетов пропущена");
        return;
    }
    let app = support::app().await;
    let index = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frontend/dist/index.html"),
    )
    .expect("index.html");
    let start = index.find("/assets/").expect("ссылка на ассет");
    let rest = &index[start..];
    let end = rest.find('"').expect("конец ссылки");
    let asset = &rest[..end];

    let response = get(&app, asset).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    let content_type = response.headers()[header::CONTENT_TYPE]
        .to_str()
        .expect("заголовок");
    assert!(
        content_type.starts_with("text/javascript") || content_type.starts_with("text/css"),
        "неожиданный тип: {content_type}"
    );
}

#[tokio::test]
async fn unknown_api_route_is_json_404() {
    let app = support::app().await;
    let response = get(&app, "/api/nope").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = body_text(response).await;
    assert!(body.contains("\"code\":\"not_found\""), "{body}");
}

#[tokio::test]
async fn unknown_file_is_404_not_index() {
    if !dist_ready() {
        eprintln!("frontend/dist не собран — проверка 404 пропущена");
        return;
    }
    let app = support::app().await;
    let response = get(&app, "/missing.js").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn readiness_reports_ready() {
    let app = support::app().await;
    let response = get(&app, "/api/health/ready").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("\"status\":\"ready\""), "{body}");
}
