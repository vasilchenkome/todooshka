use std::env;
use std::net::{AddrParseError, SocketAddr};

/// Address for local development; in a container set `BIND_ADDR=0.0.0.0:8080`.
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

/// DB file in the working directory; in a container `sqlite:///data/todooshka.db`.
pub const DEFAULT_DATABASE_URL: &str = "sqlite://todooshka.db";

/// Set to `true` when the app is behind HTTPS: the cookie then gets the `Secure` flag.
pub const COOKIE_SECURE_ENV: &str = "COOKIE_SECURE";

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("BIND_ADDR={value:?} не является адресом вида 127.0.0.1:8080: {source}")]
    InvalidBindAddr {
        value: String,
        #[source]
        source: AddrParseError,
    },

    #[error("хранилище не готово: {0}")]
    Storage(#[from] todooshka_sqlite::StorageError),
}

/// Reads the address from the environment and validates it before bind: errors surface early.
pub fn bind_addr() -> Result<SocketAddr, StartupError> {
    let value = env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned());
    value
        .parse()
        .map_err(|source| StartupError::InvalidBindAddr { value, source })
}

/// Database address: parsed and validated when connecting, before the server starts.
pub fn database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned())
}

/// Whether the session cookie needs the `Secure` flag. Off by default: LAN over HTTP.
pub fn cookie_secure() -> bool {
    matches!(
        env::var(COOKIE_SECURE_ENV)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes"
    )
}
