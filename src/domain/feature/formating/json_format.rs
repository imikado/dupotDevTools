use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::DomainResult;
use crate::domain::shared::json::to_pretty_json;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "json_format",
    section: "formating",
    title: "JSON Formatter",
    description: "Prettify or minify JSON",
};

fn parse(input: &str) -> DomainResult<serde_json::Value> {
    Ok(serde_json::from_str(input)?)
}

/// 4-space indented JSON, key order kept.
pub fn prettify(input: &str) -> DomainResult<String> {
    to_pretty_json(&parse(input)?)
}

/// JSON on a single line, without whitespace.
pub fn minify(input: &str) -> DomainResult<String> {
    Ok(serde_json::to_string(&parse(input)?)?)
}
