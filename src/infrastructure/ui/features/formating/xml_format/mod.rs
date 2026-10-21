use crate::domain::feature::formating::xml_format;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input XML",
            input_language: Some("xml"),
            output_language: Some("xml"),
            ..Default::default()
        });
        tool.add_button("Format", |t| t.report(xml_format::format(&t.input())));
        tool.into_widget()
    }
}
