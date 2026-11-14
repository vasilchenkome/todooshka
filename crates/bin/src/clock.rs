use std::time::{SystemTime, UNIX_EPOCH};

use todooshka_app::Clock;
use todooshka_domain::Timestamp;

/// The system clock: the only place where use cases learn the real time.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();

        Timestamp::from_unix_seconds(seconds)
    }
}
