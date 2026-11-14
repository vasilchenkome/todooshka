use tracing_subscriber::EnvFilter;

/// Инициализирует логи: уровень из `RUST_LOG`, формат — из `LOG_FORMAT` (`json` для контейнера).
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let json = std::env::var("LOG_FORMAT")
        .map(|value| value.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let builder = tracing_subscriber::fmt().with_env_filter(filter);

    if json {
        builder.json().init();
    } else {
        builder.init();
    }
}
