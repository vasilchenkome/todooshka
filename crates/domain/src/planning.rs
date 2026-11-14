use crate::error::DomainError;

/// Upper bound for an estimate: a personal tracker plans at most a day per task.
pub const MAX_ESTIMATE_MINUTES: i64 = 100_000;

/// Task priority: four levels, ordered from low to urgent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

impl Priority {
    pub const ALL: [Self; 4] = [Self::Low, Self::Normal, Self::High, Self::Urgent];

    pub const fn as_code(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            "urgent" => Some(Self::Urgent),
            _ => None,
        }
    }
}

/// Calendar date without a time: `YYYY-MM-DD`, compared as a string.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DueDate(String);

impl DueDate {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let value = raw.trim();
        let bytes = value.as_bytes();

        let shaped = bytes.len() == 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes
                .iter()
                .enumerate()
                .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
        if !shaped {
            return Err(DomainError::InvalidDueDate(value.to_owned()));
        }

        let year = parse_number(&bytes[0..4]);
        let month = parse_number(&bytes[5..7]);
        let day = parse_number(&bytes[8..10]);

        let valid = (1970..=2100).contains(&year)
            && (1..=12).contains(&month)
            && (1..=days_in_month(year, month)).contains(&day);
        if !valid {
            return Err(DomainError::InvalidDueDate(value.to_owned()));
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Calendar addition without time zones: `2026-12-31 + 1` is `2027-01-01`.
    pub fn plus_days(&self, days: i64) -> Option<Self> {
        let target = civil_from_days(days_from_civil(self.as_str()) + days);
        if !(1970..=2100).contains(&target.0) {
            return None;
        }
        let value = format!("{:04}-{:02}-{:02}", target.0, target.1, target.2);
        DueDate::parse(&value).ok()
    }
}

fn days_from_civil(value: &str) -> i64 {
    let year = value[0..4].parse::<i64>().expect("DueDate is validated");
    let month = value[5..7].parse::<i64>().expect("DueDate is validated");
    let day = value[8..10].parse::<i64>().expect("DueDate is validated");
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
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

fn parse_number(bytes: &[u8]) -> i64 {
    bytes
        .iter()
        .fold(0, |value, byte| value * 10 + i64::from(byte - b'0'))
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Time estimate in minutes: from 1 to `MAX_ESTIMATE_MINUTES`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EstimateMinutes(i64);

impl EstimateMinutes {
    pub fn parse(minutes: i64) -> Result<Self, DomainError> {
        if (1..=MAX_ESTIMATE_MINUTES).contains(&minutes) {
            Ok(Self(minutes))
        } else {
            Err(DomainError::InvalidEstimateMinutes)
        }
    }

    pub fn as_minutes(self) -> i64 {
        self.0
    }
}

/// Task planning: due date, priority and estimate as one cohesive set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Planning {
    due_on: Option<DueDate>,
    priority: Priority,
    estimate: Option<EstimateMinutes>,
}

impl Planning {
    pub fn new(
        due_on: Option<DueDate>,
        priority: Priority,
        estimate: Option<EstimateMinutes>,
    ) -> Self {
        Self {
            due_on,
            priority,
            estimate,
        }
    }

    pub fn due_on(&self) -> Option<&DueDate> {
        self.due_on.as_ref()
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn estimate(&self) -> Option<EstimateMinutes> {
        self.estimate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_leap_day_and_rejects_wrong_days() {
        assert_eq!(
            DueDate::parse("2028-02-29").expect("29 февраля").as_str(),
            "2028-02-29"
        );
        assert_eq!(
            DueDate::parse("2026-02-29").expect_err("2026 не високосный"),
            DomainError::InvalidDueDate("2026-02-29".to_owned())
        );
        assert_eq!(
            DueDate::parse("2026-04-31").expect_err("в апреле 30 дней"),
            DomainError::InvalidDueDate("2026-04-31".to_owned())
        );
    }

    #[test]
    fn rejects_bad_format_and_range() {
        for value in [
            "20.11.2026",
            "2026-1-05",
            "1969-12-31",
            "2101-01-01",
            "2026-13-01",
        ] {
            assert!(
                DueDate::parse(value).is_err(),
                "{value} должен быть отвергнут"
            );
        }
        assert_eq!(
            DueDate::parse(" 2026-11-20 ")
                .expect("пробелы обрезаются")
                .as_str(),
            "2026-11-20"
        );
    }

    #[test]
    fn priority_codes_are_stable() {
        for priority in Priority::ALL {
            assert_eq!(Priority::from_code(priority.as_code()), Some(priority));
        }
        assert_eq!(Priority::from_code("critical"), None);
        assert_eq!(Priority::default(), Priority::Normal);
        assert!(Priority::Urgent > Priority::High);
    }

    #[test]
    fn estimate_has_bounds() {
        assert_eq!(EstimateMinutes::parse(1).expect("минута").as_minutes(), 1);
        assert_eq!(
            EstimateMinutes::parse(MAX_ESTIMATE_MINUTES)
                .expect("верхняя граница")
                .as_minutes(),
            MAX_ESTIMATE_MINUTES
        );
        assert_eq!(
            EstimateMinutes::parse(0).expect_err("ноль нельзя"),
            DomainError::InvalidEstimateMinutes
        );
        assert!(EstimateMinutes::parse(MAX_ESTIMATE_MINUTES + 1).is_err());
    }

    #[test]
    fn adds_calendar_days_across_month_and_year_boundaries() {
        assert_eq!(
            DueDate::parse("2026-12-31")
                .expect("дата")
                .plus_days(1)
                .expect("следующий год")
                .as_str(),
            "2027-01-01"
        );
        assert_eq!(
            DueDate::parse("2028-02-28")
                .expect("дата")
                .plus_days(1)
                .expect("високосный")
                .as_str(),
            "2028-02-29"
        );
        assert_eq!(
            DueDate::parse("2026-02-28")
                .expect("дата")
                .plus_days(1)
                .expect("невисокосный")
                .as_str(),
            "2026-03-01"
        );
        assert_eq!(
            DueDate::parse("2026-11-14")
                .expect("дата")
                .plus_days(7)
                .expect("неделя")
                .as_str(),
            "2026-11-21"
        );
    }
}
