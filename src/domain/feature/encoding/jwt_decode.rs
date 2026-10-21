use base64::Engine;
use base64::alphabet::URL_SAFE;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use serde_json::{Value, json};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::shared::json::to_pretty_json;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "jwt_decode",
    section: "encoding",
    title: "JWT Decoder",
    description: "Decode JWT tokens to JSON",
};

const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(
    &URL_SAFE,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

fn decode_part(part: &str) -> DomainResult<Value> {
    Ok(serde_json::from_slice(&URL_SAFE_LENIENT.decode(part)?)?)
}

/// Decodes a JWT's header and payload (the signature is not verified) into
/// `{"header": …, "payload": …}` pretty JSON.
pub fn decode(token: &str) -> DomainResult<String> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() < 2 {
        return Err(DomainError::new(
            "not a valid JWT (expected header.payload.signature)",
        ));
    }
    to_pretty_json(&json!({
        "header": decode_part(parts[0])?,
        "payload": decode_part(parts[1])?,
    }))
}
