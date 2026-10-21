use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::DomainResult;
use crate::domain::shared::json::to_pretty_json;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "yaml_json",
    section: "convert",
    title: "YAML ↔ JSON",
    description: "Convert YAML ↔ JSON",
};

/// YAML to 4-space-indented JSON, keeping key order.
pub fn yaml_to_json(input: &str) -> DomainResult<String> {
    let value: serde_json::Value = serde_yaml_ng::from_str(input)?;
    to_pretty_json(&value)
}

/// JSON to block-style YAML, keeping key order.
pub fn json_to_yaml(input: &str) -> DomainResult<String> {
    let value: serde_json::Value = serde_json::from_str(input)?;
    Ok(serde_yaml_ng::to_string(&value)?)
}
