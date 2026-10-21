use sqlformat::{FormatOptions, Indent, QueryParams};

use crate::domain::entity::feature_entity::FeatureEntity;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "sql_format",
    section: "formating",
    title: "SQL Formatter",
    description: "Format SQL queries",
};

/// Reindents SQL with 4 spaces and upper-case keywords.
pub fn format(input: &str) -> String {
    let options = FormatOptions {
        indent: Indent::Spaces(4),
        uppercase: Some(true),
        ..Default::default()
    };
    sqlformat::format(input, &QueryParams::None, &options)
}
