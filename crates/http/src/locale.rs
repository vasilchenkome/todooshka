use axum::extract::Request;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::middleware::Next;
use axum::response::Response;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApiLocale {
    #[default]
    Ru,
    En,
}

tokio::task_local! {
    static LOCALE: ApiLocale;
}

pub fn current() -> ApiLocale {
    LOCALE.try_with(|locale| *locale).unwrap_or_default()
}

fn parse(value: &str) -> ApiLocale {
    for item in value.split(',') {
        let tag = item
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if tag.starts_with("en") {
            return ApiLocale::En;
        }
        if tag.starts_with("ru") {
            return ApiLocale::Ru;
        }
    }
    ApiLocale::Ru
}

pub async fn with_locale(request: Request, next: Next) -> Response {
    let locale = request
        .headers()
        .get(ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .map(parse)
        .unwrap_or_default();

    LOCALE.scope(locale, next.run(request)).await
}
