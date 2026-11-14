//! SPA assets: frontend builds embed them in the binary, debug builds read them from disk.
//!
//! One binary serves both the API and the UI: the container has no separate nginx.

use axum::http::StatusCode;
use axum::http::Uri;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};

use rust_embed::Embed;

/// Vite builds the UI into `frontend/dist`; a release build embeds the files in the binary.
#[derive(Embed)]
#[folder = "../../frontend/dist/"]
struct Assets;

/// Serves a file from the UI build; an unknown path without an extension is an SPA route.
pub async fn serve(uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let requested = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };

    if requested.split('/').any(|segment| segment == "..") {
        return not_found();
    }

    if let Some(file) = Assets::get(requested) {
        return respond(requested, file);
    }

    // Client routes (/projects/1) return the shell, and React renders the page.
    if !requested.contains('.')
        && let Some(index) = Assets::get("index.html")
    {
        return respond("index.html", index);
    }

    not_found()
}

fn respond(path: &str, file: rust_embed::EmbeddedFile) -> Response {
    let cache = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    (
        [(CONTENT_TYPE, mime_for(path)), (CACHE_CONTROL, cache)],
        file.data.into_owned(),
    )
        .into_response()
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "404").into_response()
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "webmanifest" => "application/manifest+json",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
