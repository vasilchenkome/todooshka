use tracing_subscriber::EnvFilter;

/// Инициализирует логи: уровень из `RUST_LOG`, по умолчанию `info`.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
