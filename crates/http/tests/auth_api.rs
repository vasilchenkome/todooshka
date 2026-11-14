mod support;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use serde_json::json;
use tower::ServiceExt;

use support::{PASSWORD, USERNAME, app, read_json, request, request_anon, send};

fn set_cookie(response: &axum::http::Response<axum::body::Body>) -> Option<String> {
    response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

#[tokio::test]
async fn health_and_login_are_public() {
    let app = app().await;

    let (health_status, _) = request_anon(&app, Method::GET, "/api/health", None).await;
    let (login_status, body) = request_anon(
        &app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "username": USERNAME, "password": PASSWORD })),
    )
    .await;

    assert_eq!(health_status, StatusCode::OK);
    assert_eq!(login_status, StatusCode::OK);
    assert_eq!(body["user"]["username"], USERNAME);
}

#[tokio::test]
async fn login_error_is_english_with_accept_language() {
    let app = app().await;
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT_LANGUAGE, "en-US,en;q=0.9")
                .body(Body::from(
                    json!({ "username": USERNAME, "password": "wrong" }).to_string(),
                ))
                .expect("запрос"),
        )
        .await
        .expect("ответ");
    let (status, body) = read_json(response).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");
    assert_eq!(body["error"]["message"], "wrong username or password");
}

#[tokio::test]
async fn login_sets_http_only_session_cookie() {
    let app = app().await;

    let response = send(
        &app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "username": USERNAME, "password": PASSWORD })),
        false,
    )
    .await;
    let status = response.status();
    let cookie = set_cookie(&response).expect("cookie выставлена");

    assert_eq!(status, StatusCode::OK);
    assert!(
        cookie.contains("HttpOnly"),
        "cookie без HttpOnly небезопасна"
    );
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.contains("Path=/"));
    assert!(
        cookie.contains("Max-Age=2592000"),
        "сессия на 30 дней: {cookie}"
    );
    assert!(
        !cookie.contains("Secure"),
        "по HTTP флаг Secure выключен до появления TLS"
    );
}

#[tokio::test]
async fn wrong_password_is_rejected_without_cookie() {
    let app = app().await;

    let response = send(
        &app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "username": USERNAME, "password": "wrong-password" })),
        false,
    )
    .await;
    let status = response.status();
    let has_cookie = set_cookie(&response).is_some();
    let (_, body) = read_json(response).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");
    assert!(!has_cookie, "при отказе cookie не выдаётся");
}

#[tokio::test]
async fn protected_routes_without_session_are_unauthorized() {
    let app = app().await;

    for uri in ["/api/projects", "/api/auth/me", "/api/projects/1/tasks"] {
        let (status, body) = request_anon(&app, Method::GET, uri, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{uri}");
        assert_eq!(body["error"]["code"], "unauthorized");
    }
}

#[tokio::test]
async fn me_returns_authenticated_user() {
    let app = app().await;

    let (status, body) = request(&app, Method::GET, "/api/auth/me", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], USERNAME);
    assert!(body["id"].is_i64());
}

#[tokio::test]
async fn logout_clears_cookie_and_invalidates_session() {
    let app = app().await;

    let response = send(&app, Method::POST, "/api/auth/logout", None, true).await;
    let status = response.status();
    let cookie = set_cookie(&response).expect("cookie очищается");
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(
        cookie.contains("Max-Age=0"),
        "cookie должна удаляться: {cookie}"
    );

    let (status, body) = request(&app, Method::GET, "/api/projects", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthorized");
}

#[tokio::test]
async fn five_failed_logins_block_the_username() {
    let app = app().await;

    for _ in 0..5 {
        let (status, _) = request_anon(
            &app,
            Method::POST,
            "/api/auth/login",
            Some(json!({ "username": USERNAME, "password": "wrong-password" })),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    let (status, body) = request_anon(
        &app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "username": USERNAME, "password": PASSWORD })),
    )
    .await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "too_many_attempts");
}
