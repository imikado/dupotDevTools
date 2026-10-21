use crate::domain::feature::convert::html_to_text;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input HTML",
            output_label: "Plain text",
            input_language: Some("html"),
            ..Default::default()
        });
        tool.add_button("Convert", |t| t.report(html_to_text::convert(&t.input())));
        tool.into_widget()
    }
}
