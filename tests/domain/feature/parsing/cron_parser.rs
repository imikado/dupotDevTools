use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use dupot_dev_tools::domain::feature::parsing::cron_parser::{EXAMPLES, explain, next_runs, parse};

fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
}

#[test]
fn all_examples_parse() {
    for (expression, _) in EXAMPLES {
        assert!(parse(expression).is_ok(), "{expression}");
    }
}

#[test]
fn rejects_invalid_expressions() {
    assert!(parse("").is_err());
    assert!(
        parse("61 * * * *")
            .unwrap_err()
            .message()
            .starts_with("Invalid cron expression")
    );
    assert!(parse("* * *").is_err());
}

#[test]
fn next_runs_every_5_minutes() {
    let cron = parse("*/5 * * * *").unwrap();
    let runs = next_runs(&cron, at(2024, 10, 21, 10, 2), 3);
    assert_eq!(
        runs,
        [
            at(2024, 10, 21, 10, 5),
            at(2024, 10, 21, 10, 10),
            at(2024, 10, 21, 10, 15)
        ]
    );
}

#[test]
fn next_runs_skip_weekends() {
    // 2024-10-25 is a Friday.
    let cron = parse("30 8 * * 1-5").unwrap();
    let runs = next_runs(&cron, at(2024, 10, 25, 9, 0), 1);
    assert_eq!(runs, [at(2024, 10, 28, 8, 30)]);
}

#[test]
fn next_runs_are_computed_in_the_local_zone() {
    let paris = FixedOffset::east_opt(2 * 3600).unwrap();
    let cron = parse("0 9 * * *").unwrap();
    let runs = next_runs(&cron, at(2024, 10, 21, 6, 0).with_timezone(&paris), 1);
    // 09:00 in UTC+2 is 07:00 UTC.
    assert_eq!(runs[0].to_utc(), at(2024, 10, 21, 7, 0));
}

#[test]
fn explain_shows_meaning_and_runs() {
    let out = explain("0 0 * * *", at(2024, 10, 21, 10, 0), &Utc, 2).unwrap();
    assert!(out.starts_with("Expression 0 0 * * *\nMeaning    "));
    assert!(out.contains("Tue 2024-10-22 00:00:00   in 14 hours"));
    assert!(out.contains("Wed 2024-10-23 00:00:00   in 1 day"));
}
