use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use ulid::Ulid;
use uuid::{Uuid, Variant};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "uuid_generator",
    section: "string",
    title: "UUID / ULID",
    description: "Generate and inspect UUIDs and ULIDs",
};

/// Upper bound for one batch, to keep the output usable.
pub const MAX_COUNT: usize = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdKind {
    /// Random.
    UuidV4,
    /// Time-ordered (Unix milliseconds + random), sortable.
    UuidV7,
    /// Time-ordered, 26 Crockford base32 characters.
    Ulid,
    Nil,
}

impl IdKind {
    pub const ALL: [IdKind; 4] = [IdKind::UuidV4, IdKind::UuidV7, IdKind::Ulid, IdKind::Nil];

    pub fn label(self) -> &'static str {
        match self {
            IdKind::UuidV4 => "UUID v4 (random)",
            IdKind::UuidV7 => "UUID v7 (time-ordered)",
            IdKind::Ulid => "ULID",
            IdKind::Nil => "Nil UUID",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IdFormat {
    pub uppercase: bool,
    /// UUIDs only: drop the dashes (32 hex digits).
    pub no_hyphens: bool,
    /// UUIDs only: wrap in `{…}`.
    pub braces: bool,
}

pub fn format_uuid(uuid: Uuid, format: IdFormat) -> String {
    let mut s = if format.no_hyphens {
        uuid.simple().to_string()
    } else {
        uuid.hyphenated().to_string()
    };
    if format.uppercase {
        s.make_ascii_uppercase();
    }
    if format.braces {
        s = format!("{{{s}}}");
    }
    s
}

fn generate_one(kind: IdKind, format: IdFormat) -> String {
    match kind {
        IdKind::UuidV4 => format_uuid(Uuid::new_v4(), format),
        IdKind::UuidV7 => format_uuid(Uuid::now_v7(), format),
        IdKind::Nil => format_uuid(Uuid::nil(), format),
        IdKind::Ulid => {
            let s = Ulid::generate().to_string();
            if format.uppercase {
                s
            } else {
                s.to_ascii_lowercase()
            }
        }
    }
}

/// `count` identifiers (clamped to 1..=MAX_COUNT), one per line.
pub fn generate(kind: IdKind, count: usize, format: IdFormat) -> String {
    (0..count.clamp(1, MAX_COUNT))
        .map(|_| generate_one(kind, format))
        .collect::<Vec<_>>()
        .join("\n")
}

fn version_name(version: usize) -> &'static str {
    match version {
        1 => "time-based (Gregorian + MAC)",
        2 => "DCE security",
        3 => "name-based (MD5)",
        4 => "random",
        5 => "name-based (SHA-1)",
        6 => "time-ordered (Gregorian)",
        7 => "time-ordered (Unix)",
        8 => "custom",
        _ => "none",
    }
}

fn variant_name(variant: Variant) -> &'static str {
    match variant {
        Variant::NCS => "NCS (reserved)",
        Variant::RFC4122 => "RFC 9562 / 4122",
        Variant::Microsoft => "Microsoft (reserved)",
        _ => "future (reserved)",
    }
}

fn format_date<Tz: TimeZone>(date: DateTime<Utc>, local: &Tz) -> String
where
    Tz::Offset: Display,
{
    format!(
        "{} ({})",
        date.format("%Y-%m-%d %H:%M:%S%.3f UTC"),
        date.with_timezone(local)
            .format("%Y-%m-%d %H:%M:%S%.3f %:z")
    )
}

/// Describes a UUID (any common notation) or a ULID: canonical form,
/// version, variant and embedded creation time when there is one.
pub fn inspect<Tz: TimeZone>(input: &str, local: &Tz) -> DomainResult<String>
where
    Tz::Offset: Display,
{
    let input = input.trim();
    if input.is_empty() {
        return Err(DomainError::new("Paste a UUID or a ULID to inspect it."));
    }
    let mut rows: Vec<(&str, String)> = Vec::new();

    if let Ok(uuid) = Uuid::try_parse(input) {
        rows.push(("Type", "UUID".into()));
        rows.push(("Canonical", uuid.hyphenated().to_string()));
        rows.push(("Hex", uuid.simple().to_string()));
        let version = uuid.get_version_num();
        rows.push(("Version", format!("{version} — {}", version_name(version))));
        rows.push(("Variant", variant_name(uuid.get_variant()).into()));
        if let Some(date) = uuid
            .get_timestamp()
            .map(|t| t.to_unix())
            .and_then(|(secs, nanos)| DateTime::from_timestamp(secs as i64, nanos))
        {
            rows.push(("Created", format_date(date, local)));
        }
    } else if let Ok(ulid) = Ulid::from_string(input) {
        rows.push(("Type", "ULID".into()));
        rows.push(("Canonical", ulid.to_string()));
        rows.push(("As UUID", Uuid::from_u128(ulid.0).hyphenated().to_string()));
        let millis = ulid.timestamp_ms() as i64;
        if let Some(date) = DateTime::from_timestamp_millis(millis) {
            rows.push(("Created", format_date(date, local)));
        }
    } else {
        return Err(DomainError::new(format!(
            "Not a valid UUID or ULID: {input}"
        )));
    }

    Ok(rows
        .iter()
        .map(|(label, value)| format!("{label:<11}{value}"))
        .collect::<Vec<_>>()
        .join("\n"))
}
