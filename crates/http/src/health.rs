use axum::Json;
use serde::Serialize;

/// Liveness-ответ: процесс жив. Проверка БД появится в `/api/health/ready`.
#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
    pub name: &'static str,
    pub version: &'static str,
}

pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        name: "todooshka",
        version: env!("CARGO_PKG_VERSION"),
    })
}
