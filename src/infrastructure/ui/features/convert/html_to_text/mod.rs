use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub const DESCRIPTION: &str = "Strip HTML tags to plain text";

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input HTML",
            output_label: "Plain text",
            input_language: Some("html"),
            ..Default::default()
        });
        tool.add_button("Convert", |t| {
            let result = html2text::from_read(t.input().as_bytes(), 80).map_err(|e| e.to_string());
            t.report("Converted", result);
        });
        tool.into_widget()
    }
}
