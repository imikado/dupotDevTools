use crate::domain::feature::formating::sql_format;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input SQL",
            input_language: Some("sql"),
            output_language: Some("sql"),
            ..Default::default()
        });
        tool.add_button("Format", |t| t.report(Ok(sql_format::format(&t.input()))));
        tool.into_widget()
    }
}
