use base64::Engine;
use base64::alphabet::URL_SAFE;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use serde_json::{Value, json};

use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};
use crate::infrastructure::ui::shared::json::to_pretty_json;

pub const DESCRIPTION: &str = "Decode JWT tokens to JSON";

const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(
    &URL_SAFE,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

pub struct Feature;

fn decode_part(part: &str) -> Result<Value, String> {
    let bytes = URL_SAFE_LENIENT.decode(part).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn decode(token: &str) -> Result<String, String> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() < 2 {
        return Err("not a valid JWT (expected header.payload.signature)".into());
    }
    to_pretty_json(&json!({
        "header": decode_part(parts[0])?,
        "payload": decode_part(parts[1])?,
    }))
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "JWT Token",
            output_label: "Decoded",
            output_language: Some("json"),
            ..Default::default()
        });
        tool.add_button("Decode", |t| t.report("Decoded", decode(&t.input())));
        tool.into_widget()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decodes_header_and_payload() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.sig";
        let out = super::decode(token).unwrap();
        assert!(out.starts_with("{\n    \"header\": {\n        \"alg\": \"HS256\""));
        assert!(out.contains("\"name\": \"John Doe\""));
        assert!(super::decode("nope").is_err());
    }
}
