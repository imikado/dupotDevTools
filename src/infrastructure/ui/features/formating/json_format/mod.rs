use crate::domain::feature::formating::json_format;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input JSON",
            input_language: Some("json"),
            output_language: Some("json"),
            ..Default::default()
        });
        tool.add_button("Format", |t| t.report(json_format::prettify(&t.input())));
        tool.add_button_with_class("Minify", None, |t| {
            t.report(json_format::minify(&t.input()))
        });
        tool.into_widget()
    }
}
