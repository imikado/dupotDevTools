use crate::domain::feature::convert::yaml_json;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

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
            t.report(yaml_json::yaml_to_json(&t.input()));
        });
        tool.add_button_with_class("JSON → YAML", None, |t| {
            t.set_input_language(Some("json"));
            t.set_output_language(Some("yaml"));
            t.report(yaml_json::json_to_yaml(&t.input()));
        });
        tool.into_widget()
    }
}
