use dupot_dev_tools::domain::feature::convert::unit_converter::{
    Quantity, describe, human_duration, human_size, iso8601, parse,
};

fn bytes(input: &str) -> f64 {
    match parse(input).unwrap() {
        Some(Quantity::Bytes(b)) => b,
        other => panic!("{input}: {other:?}"),
    }
}

fn seconds(input: &str) -> f64 {
    match parse(input).unwrap() {
        Some(Quantity::Seconds(s)) => s,
        other => panic!("{input}: {other:?}"),
    }
}

#[test]
fn parses_sizes() {
    assert_eq!(bytes("1.5 GiB"), 1_610_612_736.0);
    assert_eq!(bytes("2kB"), 2000.0);
    assert_eq!(bytes("8 bit"), 1.0);
    assert_eq!(bytes("100 Mb"), 12_500_000.0);
    assert_eq!(bytes("100 MB"), 100_000_000.0);
    assert_eq!(bytes("100 mb"), 100_000_000.0);
    assert_eq!(bytes("1,5 KiB"), 1536.0);
}

#[test]
fn parses_durations() {
    assert_eq!(seconds("1h30m"), 5400.0);
    assert_eq!(seconds("1 h 30 min 15 s"), 5415.0);
    assert_eq!(seconds("250ms"), 0.25);
    assert_eq!(seconds("2 days"), 172_800.0);
    assert_eq!(seconds("PT1H30M"), 5400.0);
    assert_eq!(seconds("P1DT0.5S"), 86_400.5);
    assert_eq!(seconds("01:30:00"), 5400.0);
    assert_eq!(seconds("1:30"), 90.0);
}

#[test]
fn bare_numbers_and_unknown_units() {
    assert_eq!(parse("42").unwrap(), None);
    assert!(parse("").is_err());
    assert!(parse("12 parsecs").is_err());
    assert!(parse("1h30").is_err());
    assert!(parse("P").is_err());
}

#[test]
fn formats_for_humans() {
    assert_eq!(human_size(1_610_612_736.0, true), "1.5 GiB");
    assert_eq!(human_size(1_500_000.0, false), "1.5 MB");
    assert_eq!(human_size(12.0, true), "12 B");
    assert_eq!(human_duration(5415.0), "1h 30m 15s");
    assert_eq!(human_duration(90_061.5), "1d 1h 1m 1.5s");
    assert_eq!(human_duration(0.25), "250ms");
    assert_eq!(human_duration(0.0), "0s");
    assert_eq!(iso8601(5400.0), "PT1H30M");
    assert_eq!(iso8601(86_400.0), "P1D");
    assert_eq!(iso8601(0.0), "PT0S");
}

#[test]
fn describes_conversions() {
    let size = describe("1 GiB").unwrap();
    assert!(size.contains("MiB  1024"), "{size}");
    assert!(size.contains("GB  1.073742"), "{size}");
    let duration = describe("1h30m").unwrap();
    assert!(duration.contains("ISO 8601  PT1H30M"), "{duration}");
    assert!(duration.contains("min  90"), "{duration}");
    let bare = describe("3600").unwrap();
    assert!(
        bare.contains("As bytes") && bare.contains("Human     1h"),
        "{bare}"
    );
}
