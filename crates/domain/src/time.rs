/// A point in time in unix seconds: the domain relies on no date or clock crate.
///
/// Time comes from the `Clock` port at the use-case level, so the rules
/// and the tests stay deterministic.
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
