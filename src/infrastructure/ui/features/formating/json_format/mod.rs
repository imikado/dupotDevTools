use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};
use crate::infrastructure::ui::shared::json::to_pretty_json;

pub const DESCRIPTION: &str = "Prettify or minify JSON";

pub struct Feature;

fn format(input: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    to_pretty_json(&value)
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input JSON",
            input_language: Some("json"),
            output_language: Some("json"),
            ..Default::default()
        });
        tool.add_button("Format", |t| t.report("Formatted", format(&t.input())));
        tool.into_widget()
    }
}
