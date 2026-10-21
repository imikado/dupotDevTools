use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use dupot_dev_tools::domain::feature::convert::timestamp::{
    TimestampUnit, describe, parse, relative,
};

fn paris() -> FixedOffset {
    FixedOffset::east_opt(2 * 3600).unwrap()
}

fn at(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap()
}

#[test]
fn detects_units_from_magnitude() {
    assert_eq!(TimestampUnit::detect(1_729_540_000), TimestampUnit::Seconds);
    assert_eq!(
        TimestampUnit::detect(1_729_540_000_123),
        TimestampUnit::Milliseconds
    );
    assert_eq!(
        TimestampUnit::detect(1_729_540_000_123_456),
        TimestampUnit::Microseconds
    );
    assert_eq!(
        TimestampUnit::detect(1_729_540_000_123_456_789),
        TimestampUnit::Nanoseconds
    );
    assert_eq!(TimestampUnit::detect(-86_400), TimestampUnit::Seconds);
}

#[test]
fn parses_every_unit_to_the_same_instant() {
    for input in [
        "1729540000",
        "1729540000000",
        "1729540000000000",
        "1729540000000000000",
    ] {
        let (date, _) = parse(input, &Utc).unwrap();
        assert_eq!(date, at(1_729_540_000), "{input}");
    }
}

#[test]
fn parses_fractional_seconds() {
    let (date, unit) = parse("1729540000.5", &Utc).unwrap();
    assert_eq!(date.timestamp_millis(), 1_729_540_000_500);
    assert_eq!(unit, Some(TimestampUnit::Seconds));
}

#[test]
fn parses_dates_with_and_without_offset() {
    let utc = at(1_729_540_000); // 2024-10-21 19:46:40 UTC
    for input in [
        "2024-10-21T19:46:40Z",
        "2024-10-21T21:46:40+02:00",
        "Mon, 21 Oct 2024 19:46:40 +0000",
    ] {
        assert_eq!(parse(input, &paris()).unwrap(), (utc, None), "{input}");
    }
    // No offset: read in the local zone (UTC+2 here).
    for input in [
        "2024-10-21 21:46:40",
        "2024-10-21T21:46:40",
        "21/10/2024 21:46:40",
    ] {
        assert_eq!(parse(input, &paris()).unwrap().0, utc, "{input}");
    }
    let midnight = paris().with_ymd_and_hms(2024, 10, 21, 0, 0, 0).unwrap();
    assert_eq!(parse("2024-10-21", &paris()).unwrap().0, midnight);
}

#[test]
fn rejects_empty_and_garbage() {
    assert!(parse("  ", &Utc).is_err());
    assert!(
        parse("yesterday-ish", &Utc)
            .unwrap_err()
            .message()
            .contains("Examples")
    );
}

#[test]
fn relative_times() {
    let now = at(1_000_000_000);
    assert_eq!(relative(now, now), "just now");
    assert_eq!(relative(at(1_000_000_000 - 90), now), "1 minute ago");
    assert_eq!(relative(at(1_000_000_000 - 3 * 3600), now), "3 hours ago");
    assert_eq!(relative(at(1_000_000_000 + 2 * 86_400), now), "in 2 days");
    assert_eq!(
        relative(at(1_000_000_000 - 2 * 31_536_000), now),
        "2 years ago"
    );
}

#[test]
fn describe_lists_all_representations() {
    let out = describe("1729540000", at(1_729_540_000 + 3600), &paris()).unwrap();
    assert!(out.contains("Detected   Unix timestamp (seconds)"));
    assert!(out.contains("UTC        2024-10-21 19:46:40.000 UTC"));
    assert!(out.contains("Local      2024-10-21 21:46:40.000 +02:00"));
    assert!(out.contains("ISO 8601   2024-10-21T19:46:40Z"));
    assert!(out.contains("Unix (ms)  1729540000000"));
    assert!(out.contains("Relative   1 hour ago"));
    assert!(out.contains("Calendar   Monday, week 43, day 295 of the year"));
}
