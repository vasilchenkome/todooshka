/// Момент времени в unix-секундах: домен не зависит от библиотек дат и часов.
///
/// Время приходит из порта `Clock` на уровне сценариев, поэтому правила
/// и тесты остаются детерминированными.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const fn from_unix_seconds(seconds: i64) -> Self {
        Self(seconds)
    }

    pub const fn as_unix_seconds(self) -> i64 {
        self.0
    }
}
