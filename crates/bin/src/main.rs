//! Composition root: конфигурация, сборка зависимостей и запуск сервера.

mod config;
mod logging;

use std::process::ExitCode;

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
    let addr = config::bind_addr()?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| config::StartupError::Bind { addr, source })?;

    tracing::info!(%addr, version = env!("CARGO_PKG_VERSION"), "todooshka запущен");

    axum::serve(listener, todooshka_http::router())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(config::StartupError::Serve)
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(%err, "не удалось подписаться на Ctrl+C");
        return;
    }
    tracing::info!("получен сигнал остановки, завершаю работу");
}
