use std::fmt::Display;

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "timestamp",
    section: "convert",
    title: "Timestamp ↔ Date",
    description: "Convert Unix timestamps to dates and back",
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampUnit {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

impl TimestampUnit {
    pub fn label(self) -> &'static str {
        match self {
            TimestampUnit::Seconds => "seconds",
            TimestampUnit::Milliseconds => "milliseconds",
            TimestampUnit::Microseconds => "microseconds",
            TimestampUnit::Nanoseconds => "nanoseconds",
        }
    }

    /// Guesses the unit from the magnitude: values up to 11 digits are
    /// seconds (until year 5138), then milli-, micro- and nanoseconds.
    pub fn detect(value: i64) -> Self {
        match value.unsigned_abs() {
            0..100_000_000_000 => TimestampUnit::Seconds,
            100_000_000_000..100_000_000_000_000 => TimestampUnit::Milliseconds,
            100_000_000_000_000..100_000_000_000_000_000 => TimestampUnit::Microseconds,
            _ => TimestampUnit::Nanoseconds,
        }
    }
}

/// Date-time layouts accepted without a time zone (read in the local zone).
const NAIVE_DATETIME_FORMATS: [&str; 6] = [
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%dT%H:%M:%S%.f",
    "%Y-%m-%d %H:%M",
    "%Y-%m-%dT%H:%M",
    "%Y/%m/%d %H:%M:%S",
    "%d/%m/%Y %H:%M:%S",
];
const NAIVE_DATE_FORMATS: [&str; 3] = ["%Y-%m-%d", "%Y/%m/%d", "%d/%m/%Y"];

fn from_unit(value: i64, unit: TimestampUnit) -> Option<DateTime<Utc>> {
    match unit {
        TimestampUnit::Seconds => DateTime::from_timestamp(value, 0),
        TimestampUnit::Milliseconds => DateTime::from_timestamp_millis(value),
        TimestampUnit::Microseconds => DateTime::from_timestamp_micros(value),
        TimestampUnit::Nanoseconds => Some(DateTime::from_timestamp_nanos(value)),
    }
}

/// Parses a Unix timestamp (unit auto-detected, fractional seconds allowed)
/// or a date. Dates without an explicit offset are read in `local`.
pub fn parse<Tz: TimeZone>(
    input: &str,
    local: &Tz,
) -> DomainResult<(DateTime<Utc>, Option<TimestampUnit>)> {
    let input = input.trim();
    if input.is_empty() {
        return Err(DomainError::new("Enter a timestamp or a date."));
    }
    let out_of_range = || DomainError::new("Timestamp out of range.");

    if let Ok(value) = input.parse::<i64>() {
        let unit = TimestampUnit::detect(value);
        return Ok((from_unit(value, unit).ok_or_else(out_of_range)?, Some(unit)));
    }
    if let Ok(value) = input.parse::<f64>() {
        let nanos = (value * 1e9).round();
        if !nanos.is_finite() || nanos.abs() > i64::MAX as f64 {
            return Err(out_of_range());
        }
        let date = DateTime::from_timestamp_nanos(nanos as i64);
        return Ok((date, Some(TimestampUnit::Seconds)));
    }
    if let Ok(date) = DateTime::parse_from_rfc3339(input) {
        return Ok((date.to_utc(), None));
    }
    if let Ok(date) = DateTime::parse_from_rfc2822(input) {
        return Ok((date.to_utc(), None));
    }
    let naive = NAIVE_DATETIME_FORMATS
        .iter()
        .find_map(|f| NaiveDateTime::parse_from_str(input, f).ok())
        .or_else(|| {
            NAIVE_DATE_FORMATS
                .iter()
                .find_map(|f| NaiveDate::parse_from_str(input, f).ok())
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        });
    if let Some(naive) = naive {
        let date = local
            .from_local_datetime(&naive)
            .earliest()
            .ok_or_else(|| DomainError::new("This local time does not exist (DST change)."))?;
        return Ok((date.to_utc(), None));
    }
    Err(DomainError::new(format!(
        "Unrecognized timestamp or date: {input}\n\
         Examples: 1729540000, 1729540000000, 2024-10-21 21:46:40, 2024-10-21T19:46:40Z"
    )))
}

/// "3 hours ago", "in 2 days", "just now".
pub fn relative(date: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let delta = now.signed_duration_since(date).num_seconds();
    let secs = delta.unsigned_abs();
    if secs < 45 {
        return "just now".into();
    }
    let (value, unit) = match secs {
        0..3_600 => (secs / 60, "minute"),
        3_600..86_400 => (secs / 3_600, "hour"),
        86_400..2_592_000 => (secs / 86_400, "day"),
        2_592_000..31_536_000 => (secs / 2_592_000, "month"),
        _ => (secs / 31_536_000, "year"),
    };
    let value = value.max(1);
    let plural = if value == 1 { "" } else { "s" };
    if delta > 0 {
        format!("{value} {unit}{plural} ago")
    } else {
        format!("in {value} {unit}{plural}")
    }
}

/// Every useful representation of `input`, one per line.
pub fn describe<Tz: TimeZone>(input: &str, now: DateTime<Utc>, local: &Tz) -> DomainResult<String>
where
    Tz::Offset: Display,
{
    let (date, unit) = parse(input, local)?;
    let local_date = date.with_timezone(local);
    let source = match unit {
        Some(unit) => format!("Unix timestamp ({})", unit.label()),
        None => "Date".to_string(),
    };
    let rows = [
        ("Detected", source),
        ("UTC", date.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()),
        (
            "Local",
            local_date.format("%Y-%m-%d %H:%M:%S%.3f %:z").to_string(),
        ),
        (
            "ISO 8601",
            date.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
        ),
        ("RFC 2822", date.to_rfc2822()),
        ("Unix (s)", date.timestamp().to_string()),
        ("Unix (ms)", date.timestamp_millis().to_string()),
        ("Relative", relative(date, now)),
        (
            "Calendar",
            format!(
                "{}, week {}, day {} of the year",
                local_date.format("%A"),
                local_date.iso_week().week(),
                local_date.ordinal()
            ),
        ),
    ];
    Ok(rows
        .iter()
        .map(|(label, value)| format!("{label:<11}{value}"))
        .collect::<Vec<_>>()
        .join("\n"))
}
