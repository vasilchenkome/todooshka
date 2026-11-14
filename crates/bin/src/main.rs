//! Composition root: конфигурация, сборка зависимостей и запуск сервера.

mod clock;
mod config;
mod logging;

use std::process::ExitCode;
use std::sync::Arc;

use todooshka_app::{ProjectRepository, ProjectService};
use todooshka_http::AppState;
use todooshka_sqlite::SqliteProjects;

use crate::clock::SystemClock;

#[tokio::main]
async fn main() -> ExitCode {
    logging::init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), config::StartupError> {
    let database_url = config::database_url();
    let pool = todooshka_sqlite::connect(&database_url).await?;
    todooshka_sqlite::migrate(&pool).await?;
    tracing::info!("миграции применены");

    let repository: Arc<dyn ProjectRepository> = Arc::new(SqliteProjects::new(pool.clone()));
    let projects = ProjectService::new(repository, Arc::new(SystemClock));
    let state = AppState {
        projects: Arc::new(projects),
    };

    let addr = config::bind_addr()?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| config::StartupError::Bind { addr, source })?;

    tracing::info!(%addr, version = env!("CARGO_PKG_VERSION"), "todooshka запущен");

    axum::serve(listener, todooshka_http::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(config::StartupError::Serve)?;

    pool.close().await;
    Ok(())
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(%err, "не удалось подписаться на Ctrl+C");
        return;
    }
    tracing::info!("получен сигнал остановки, завершаю работу");
}
