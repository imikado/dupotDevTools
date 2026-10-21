use crate::domain::feature::encoding::jwt_decode;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "JWT Token",
            output_label: "Decoded",
            output_language: Some("json"),
            ..Default::default()
        });
        tool.add_button("Decode", |t| t.report(jwt_decode::decode(&t.input())));
        tool.into_widget()
    }
}
