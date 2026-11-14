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

    /// UTC timestamp for export files; no time-zone database is involved.
    pub fn as_rfc3339(self) -> String {
        let days = self.0.div_euclid(86_400);
        let seconds = self.0.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        let hour = seconds / 3_600;
        let minute = (seconds % 3_600) / 60;
        let second = seconds % 60;
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    }
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    (year + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_utc_timestamps() {
        assert_eq!(
            Timestamp::from_unix_seconds(0).as_rfc3339(),
            "1970-01-01T00:00:00Z"
        );
        assert_eq!(
            Timestamp::from_unix_seconds(1_700_000_000).as_rfc3339(),
            "2023-11-14T22:13:20Z"
        );
        assert_eq!(
            Timestamp::from_unix_seconds(1_709_164_800).as_rfc3339(),
            "2024-02-29T00:00:00Z"
        );
    }
}
