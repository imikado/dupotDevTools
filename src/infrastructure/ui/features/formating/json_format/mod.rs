use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};
use crate::infrastructure::ui::shared::json::to_pretty_json;

pub const TITLE: &str = "JSON Formatter";
pub const DESCRIPTION: &str = "Prettify or minify JSON";

pub struct Feature;

fn parse(input: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(input).map_err(|e| e.to_string())
}

fn format(input: &str) -> Result<String, String> {
    to_pretty_json(&parse(input)?)
}

fn minify(input: &str) -> Result<String, String> {
    serde_json::to_string(&parse(input)?).map_err(|e| e.to_string())
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input JSON",
            input_language: Some("json"),
            output_language: Some("json"),
            ..Default::default()
        });
        tool.add_button("Format", |t| t.report(format(&t.input())));
        tool.add_button_with_class("Minify", None, |t| t.report(minify(&t.input())));
        tool.into_widget()
    }
}
