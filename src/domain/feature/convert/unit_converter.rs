use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::shared::format::{number, rows};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "unit_converter",
    section: "convert",
    title: "Unit Converter",
    description: "Convert data sizes (bytes, KiB, Mbit…) and durations (ms, 1h30m, PT2H…)",
};

pub const EXAMPLES: [&str; 6] = [
    "1.5 GiB", "250 Mbit", "1h30m", "90 min", "PT2H15M", "01:30:00",
];

/// Size units and their value in bytes.
const SIZE_UNITS: [(&str, f64); 21] = [
    ("B", 1.0),
    ("byte", 1.0),
    ("bytes", 1.0),
    ("kB", 1e3),
    ("KB", 1e3),
    ("MB", 1e6),
    ("GB", 1e9),
    ("TB", 1e12),
    ("PB", 1e15),
    ("KiB", 1024.0),
    ("MiB", 1_048_576.0),
    ("GiB", 1_073_741_824.0),
    ("TiB", 1_099_511_627_776.0),
    ("PiB", 1_125_899_906_842_624.0),
    ("b", 0.125),
    ("bit", 0.125),
    ("bits", 0.125),
    ("kbit", 125.0),
    ("Mbit", 125_000.0),
    ("Gbit", 125_000_000.0),
    ("Tbit", 125_000_000_000.0),
];

/// Bit-rate style abbreviations (`Kb`, `Mb`…): exact case only.
const BIT_ABBREVIATIONS: [(&str, f64); 4] = [
    ("Kb", 125.0),
    ("Mb", 125_000.0),
    ("Gb", 125_000_000.0),
    ("Tb", 125_000_000_000.0),
];

/// Duration units and their value in seconds.
const DURATION_UNITS: [(&str, f64); 31] = [
    ("ns", 1e-9),
    ("us", 1e-6),
    ("µs", 1e-6),
    ("μs", 1e-6),
    ("ms", 1e-3),
    ("s", 1.0),
    ("sec", 1.0),
    ("secs", 1.0),
    ("second", 1.0),
    ("seconds", 1.0),
    ("m", 60.0),
    ("min", 60.0),
    ("mins", 60.0),
    ("minute", 60.0),
    ("minutes", 60.0),
    ("h", 3600.0),
    ("hr", 3600.0),
    ("hrs", 3600.0),
    ("hour", 3600.0),
    ("hours", 3600.0),
    ("d", 86_400.0),
    ("day", 86_400.0),
    ("days", 86_400.0),
    ("w", 604_800.0),
    ("wk", 604_800.0),
    ("week", 604_800.0),
    ("weeks", 604_800.0),
    ("y", YEAR),
    ("yr", YEAR),
    ("year", YEAR),
    ("years", YEAR),
];

/// Julian year, 365.25 days.
const YEAR: f64 = 31_557_600.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Quantity {
    Bytes(f64),
    Seconds(f64),
}

fn size_unit(unit: &str) -> Option<f64> {
    let exact = SIZE_UNITS
        .iter()
        .chain(&BIT_ABBREVIATIONS)
        .find(|(name, _)| *name == unit);
    // Any case is accepted, except for the `b` (bit) / `B` (byte) pair.
    let loose = || {
        SIZE_UNITS
            .iter()
            .filter(|(name, _)| *name != "b")
            .find(|(name, _)| name.eq_ignore_ascii_case(unit))
    };
    exact.or_else(loose).map(|(_, factor)| *factor)
}

fn duration_unit(unit: &str) -> Option<f64> {
    let unit = unit.to_lowercase();
    DURATION_UNITS
        .iter()
        .find(|(name, _)| *name == unit)
        .map(|(_, factor)| *factor)
}

/// Drops `_` digit separators and reads `,` as the decimal point.
fn clean_number(text: &str) -> String {
    text.replace('_', "").replace(',', ".")
}

/// Splits "1h 30.5min" into [(1, "h"), (30.5, "min")].
fn number_unit_pairs(text: &str) -> Option<Vec<(f64, &str)>> {
    let mut pairs = Vec::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        let num_end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '_' || c == ','))
            .unwrap_or(rest.len());
        let value: f64 = clean_number(&rest[..num_end]).parse().ok()?;
        rest = rest[num_end..].trim_start();
        let unit_end = rest
            .find(|c: char| c.is_ascii_digit() || c.is_whitespace())
            .unwrap_or(rest.len());
        pairs.push((value, &rest[..unit_end]));
        rest = rest[unit_end..].trim_start();
    }
    (!pairs.is_empty()).then_some(pairs)
}

/// ISO 8601 duration: `P1DT2H30M`, `PT0.5S`, `P2W`.
fn parse_iso8601(text: &str) -> Option<f64> {
    let body = text.strip_prefix(['P', 'p'])?;
    let (date, time) = match body.split_once(['T', 't']) {
        Some((d, t)) => (d, Some(t)),
        None => (body, None),
    };
    let sum = |part: &str, units: &[(char, f64)]| -> Option<f64> {
        let mut total = 0.0;
        let mut number = String::new();
        for c in part.chars() {
            if c.is_ascii_digit() || c == '.' || c == ',' {
                number.push(if c == ',' { '.' } else { c });
            } else {
                let (_, factor) = units.iter().find(|(u, _)| *u == c.to_ascii_uppercase())?;
                total += number.parse::<f64>().ok()? * factor;
                number.clear();
            }
        }
        number.is_empty().then_some(total)
    };
    let date_secs = sum(
        date,
        &[
            ('Y', YEAR),
            ('M', YEAR / 12.0),
            ('W', 604_800.0),
            ('D', 86_400.0),
        ],
    )?;
    let time_secs = match time {
        Some("") => return None,
        Some(t) => sum(t, &[('H', 3600.0), ('M', 60.0), ('S', 1.0)])?,
        None => 0.0,
    };
    (!body.is_empty()).then_some(date_secs + time_secs)
}

/// Clock notation: `mm:ss`, `hh:mm:ss`, `d:hh:mm:ss` (fractional seconds ok).
fn parse_clock(text: &str) -> Option<f64> {
    let parts: Vec<&str> = text.split(':').collect();
    if !(2..=4).contains(&parts.len()) {
        return None;
    }
    let factors = [1.0, 60.0, 3600.0, 86_400.0];
    parts
        .iter()
        .rev()
        .zip(factors)
        .try_fold(0.0, |total, (part, factor)| {
            Some(total + part.trim().parse::<f64>().ok()? * factor)
        })
}

/// Reads a size (`1.5 GiB`, `100Mbit`) or a duration (`1h30m`, `PT2H`,
/// `01:30:00`). A bare number is `None`: the unit decides the quantity.
pub fn parse(input: &str) -> DomainResult<Option<Quantity>> {
    let text = input.trim();
    if text.is_empty() {
        return Err(DomainError::new("Enter a size or a duration."));
    }
    if clean_number(text).parse::<f64>().is_ok() {
        return Ok(None);
    }
    if let Some(seconds) = parse_iso8601(text).or_else(|| parse_clock(text)) {
        return Ok(Some(Quantity::Seconds(seconds)));
    }
    let unknown = || {
        DomainError::new(format!(
            "Unrecognized size or duration: {text}\nExamples: {}",
            EXAMPLES.join(", ")
        ))
    };
    let pairs = number_unit_pairs(text).ok_or_else(unknown)?;
    if let [(value, unit)] = pairs[..]
        && let Some(factor) = size_unit(unit)
    {
        return Ok(Some(Quantity::Bytes(value * factor)));
    }
    let seconds = pairs.iter().try_fold(0.0, |total, (value, unit)| {
        duration_unit(unit).map(|factor| total + value * factor)
    });
    seconds
        .map(|s| Some(Quantity::Seconds(s)))
        .ok_or_else(unknown)
}

/// "1.5 GiB" style, in binary (`binary`) or decimal units.
pub fn human_size(bytes: f64, binary: bool) -> String {
    let (base, units) = if binary {
        (1024.0, ["B", "KiB", "MiB", "GiB", "TiB", "PiB"])
    } else {
        (1000.0, ["B", "kB", "MB", "GB", "TB", "PB"])
    };
    let mut value = bytes;
    let mut unit = 0;
    while value.abs() >= base && unit < units.len() - 1 {
        value /= base;
        unit += 1;
    }
    format!("{} {}", number(value, 2), units[unit])
}

/// "1d 2h 3m 4.5s"; sub-second durations in ms, µs or ns.
pub fn human_duration(seconds: f64) -> String {
    let abs = seconds.abs();
    let sign = if seconds < 0.0 { "-" } else { "" };
    if abs < 1.0 {
        let (value, unit) = match abs {
            0.0 => (0.0, "s"),
            a if a < 1e-6 => (a * 1e9, "ns"),
            a if a < 1e-3 => (a * 1e6, "µs"),
            a => (a * 1e3, "ms"),
        };
        return format!("{sign}{}{unit}", number(value, 3));
    }
    let mut rest = abs;
    let mut parts = Vec::new();
    for (unit, size) in [("d", 86_400.0), ("h", 3600.0), ("m", 60.0)] {
        let count = (rest / size).floor();
        if count >= 1.0 {
            parts.push(format!("{count}{unit}"));
            rest -= count * size;
        }
    }
    let secs = number(rest, 3);
    if secs != "0" {
        parts.push(format!("{secs}s"));
    }
    format!("{sign}{}", parts.join(" "))
}

/// ISO 8601 duration, days at most (no years or months, which vary).
pub fn iso8601(seconds: f64) -> String {
    let mut rest = seconds.abs();
    let days = (rest / 86_400.0).floor();
    rest -= days * 86_400.0;
    let hours = (rest / 3600.0).floor();
    rest -= hours * 3600.0;
    let minutes = (rest / 60.0).floor();
    rest -= minutes * 60.0;
    let mut out = String::from(if seconds < 0.0 { "-P" } else { "P" });
    if days > 0.0 {
        out.push_str(&format!("{days}D"));
    }
    let secs = number(rest, 3);
    if hours > 0.0 || minutes > 0.0 || secs != "0" || days == 0.0 {
        out.push('T');
        if hours > 0.0 {
            out.push_str(&format!("{hours}H"));
        }
        if minutes > 0.0 {
            out.push_str(&format!("{minutes}M"));
        }
        if secs != "0" || (hours == 0.0 && minutes == 0.0) {
            out.push_str(&format!("{secs}S"));
        }
    }
    out
}

fn clock(seconds: f64) -> String {
    let total = seconds.abs();
    let whole = total.floor() as u64;
    let millis = ((total - whole as f64) * 1000.0).round() as u64;
    let (h, m, s) = (whole / 3600, whole / 60 % 60, whole % 60);
    let sign = if seconds < 0.0 { "-" } else { "" };
    if millis > 0 {
        format!("{sign}{h:02}:{m:02}:{s:02}.{millis:03}")
    } else {
        format!("{sign}{h:02}:{m:02}:{s:02}")
    }
}

fn describe_size(bytes: f64) -> String {
    let column = |units: &[&str]| {
        units
            .iter()
            .map(|u| {
                (
                    u.to_string(),
                    number(bytes / size_unit(u).unwrap_or(1.0), 6),
                )
            })
            .collect::<Vec<_>>()
    };
    format!(
        "{}\n\nDecimal (SI)\n{}\n\nBinary (IEC)\n{}\n\nBits\n{}",
        rows(&[
            ("Bytes", number(bytes, 3)),
            (
                "Human",
                format!("{} · {}", human_size(bytes, true), human_size(bytes, false))
            ),
        ]),
        rows(&column(&["kB", "MB", "GB", "TB", "PB"])),
        rows(&column(&["KiB", "MiB", "GiB", "TiB", "PiB"])),
        rows(&column(&["bit", "kbit", "Mbit", "Gbit", "Tbit"])),
    )
}

fn describe_duration(seconds: f64) -> String {
    let units = [
        ("ns", 1e-9),
        ("µs", 1e-6),
        ("ms", 1e-3),
        ("s", 1.0),
        ("min", 60.0),
        ("h", 3600.0),
        ("d", 86_400.0),
        ("w", 604_800.0),
        ("y", YEAR),
    ];
    format!(
        "{}\n\nIn units\n{}",
        rows(&[
            ("Human", human_duration(seconds)),
            ("Clock", clock(seconds)),
            ("ISO 8601", iso8601(seconds)),
        ]),
        rows(
            &units
                .iter()
                .map(|(u, f)| (u.to_string(), number(seconds / f, 6)))
                .collect::<Vec<_>>()
        ),
    )
}

/// Every conversion of `input`. A bare number is shown both as bytes and as
/// seconds.
pub fn describe(input: &str) -> DomainResult<String> {
    Ok(match parse(input)? {
        Some(Quantity::Bytes(bytes)) => describe_size(bytes),
        Some(Quantity::Seconds(seconds)) => describe_duration(seconds),
        None => {
            let value: f64 = clean_number(input.trim()).parse()?;
            format!(
                "No unit: showing {value} as bytes, then as seconds.\n\n\
                 ── As bytes ──\n{}\n\n── As seconds ──\n{}",
                describe_size(value),
                describe_duration(value)
            )
        }
    })
}
