use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use croner::Cron;
use croner::parser::{CronParser, Seconds};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::feature::convert::timestamp::relative;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "cron_parser",
    section: "parsing",
    title: "Cron Parser",
    description: "Explain cron expressions and list next runs",
};

/// Common schedules, shown as presets: (expression, meaning).
pub const EXAMPLES: [(&str, &str); 8] = [
    ("*/5 * * * *", "Every 5 minutes"),
    ("0 * * * *", "Every hour"),
    ("0 0 * * *", "Every day at midnight"),
    ("30 8 * * 1-5", "Weekdays at 08:30"),
    ("0 9 * * MON", "Every Monday at 09:00"),
    ("0 0 1 * *", "First day of the month"),
    ("0 0 1 1 *", "Every year on January 1st"),
    (
        "0 */15 9-17 * * *",
        "Every 15 minutes from 09:00 to 17:45 (6 fields, with seconds)",
    ),
];

/// Parses a 5-field (minute hour day month weekday) or 6-field (seconds
/// first) cron expression; also accepts `@daily`-style nicknames.
pub fn parse(expression: &str) -> DomainResult<Cron> {
    let expression = expression.trim();
    if expression.is_empty() {
        return Err(DomainError::new(
            "Enter a cron expression, e.g. */5 * * * *",
        ));
    }
    CronParser::builder()
        .seconds(Seconds::Optional)
        .build()
        .parse(expression)
        .map_err(|e| DomainError::new(format!("Invalid cron expression: {e}")))
}

/// The next `count` run times strictly after `after`.
pub fn next_runs<Tz: TimeZone>(
    cron: &Cron,
    after: DateTime<Tz>,
    count: usize,
) -> Vec<DateTime<Tz>> {
    cron.iter_after(after).take(count).collect()
}

/// Expression, plain-English meaning and the next `count` runs in `local`.
pub fn explain<Tz: TimeZone>(
    expression: &str,
    now: DateTime<Utc>,
    local: &Tz,
    count: usize,
) -> DomainResult<String>
where
    Tz::Offset: Display,
{
    let cron = parse(expression)?;
    let mut out = format!(
        "{:<11}{}\n{:<11}{}\n\nNext runs ({})",
        "Expression",
        expression.trim(),
        "Meaning",
        cron.describe(),
        local.from_utc_datetime(&now.naive_utc()).format("%:z")
    );
    let runs = next_runs(&cron, now.with_timezone(local), count);
    if runs.is_empty() {
        out.push_str("\n  never");
    }
    for run in runs {
        out.push_str(&format!(
            "\n  {}   {}",
            run.format("%a %Y-%m-%d %H:%M:%S"),
            relative(run.to_utc(), now)
        ));
    }
    Ok(out)
}
