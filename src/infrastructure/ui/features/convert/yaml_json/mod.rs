use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};
use crate::infrastructure::ui::shared::json::to_pretty_json;

pub const TITLE: &str = "YAML ↔ JSON";
pub const DESCRIPTION: &str = "Convert YAML ↔ JSON";

pub struct Feature;

fn yaml_to_json(input: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_yaml_ng::from_str(input).map_err(|e| e.to_string())?;
    to_pretty_json(&value)
}

fn json_to_yaml(input: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    serde_yaml_ng::to_string(&value).map_err(|e| e.to_string())
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_language: Some("yaml"),
            output_language: Some("json"),
            ..Default::default()
        });
        tool.add_button("YAML → JSON", |t| {
            t.set_input_language(Some("yaml"));
            t.set_output_language(Some("json"));
            t.report(yaml_to_json(&t.input()));
        });
        tool.add_button_with_class("JSON → YAML", None, |t| {
            t.set_input_language(Some("json"));
            t.set_output_language(Some("yaml"));
            t.report(json_to_yaml(&t.input()));
        });
        tool.into_widget()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trip_keeps_order() {
        let json = super::yaml_to_json("b: 1\na: [x, é]\n").unwrap();
        assert_eq!(
            json,
            "{\n    \"b\": 1,\n    \"a\": [\n        \"x\",\n        \"é\"\n    ]\n}"
        );
        assert_eq!(super::json_to_yaml(&json).unwrap(), "b: 1\na:\n- x\n- é\n");
    }
}
