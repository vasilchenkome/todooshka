use std::env;
use std::io;
use std::net::{AddrParseError, SocketAddr};

/// Адрес для локальной разработки; в контейнере задаётся `BIND_ADDR=0.0.0.0:8080`.
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

/// Файл базы в рабочей директории; в контейнере — `sqlite:///data/todooshka.db`.
pub const DEFAULT_DATABASE_URL: &str = "sqlite://todooshka.db";

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("BIND_ADDR={value:?} не является адресом вида 127.0.0.1:8080: {source}")]
    InvalidBindAddr {
        value: String,
        #[source]
        source: AddrParseError,
    },

    #[error("не удалось занять {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: io::Error,
    },

    #[error("хранилище не готово: {0}")]
    Storage(#[from] todooshka_sqlite::StorageError),

    #[error("сервер остановился с ошибкой: {0}")]
    Serve(#[source] io::Error),
}

/// Читает адрес из окружения и валидирует его до bind: ошибка конфига видна сразу.
pub fn bind_addr() -> Result<SocketAddr, StartupError> {
    let value = env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned());
    value
        .parse()
        .map_err(|source| StartupError::InvalidBindAddr { value, source })
}

/// Адрес базы: разбирается и проверяется при подключении, до старта сервера.
pub fn database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned())
}
