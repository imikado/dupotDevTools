use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::DomainResult;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "base64_encoding",
    section: "encoding",
    title: "Base64",
    description: "Encode / decode Base64",
};

pub fn encode(input: &str) -> String {
    STANDARD.encode(input)
}

/// Decodes standard Base64 (surrounding whitespace ignored) into UTF-8 text.
pub fn decode(input: &str) -> DomainResult<String> {
    Ok(String::from_utf8(STANDARD.decode(input.trim())?)?)
}
