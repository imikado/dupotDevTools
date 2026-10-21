use crate::domain::feature::encoding::base64_encoding as logic;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options::default());
        tool.add_button("Encode", |t| t.report(Ok(logic::encode(&t.input()))));
        tool.add_button_with_class("Decode", None, |t| t.report(logic::decode(&t.input())));
        tool.into_widget()
    }
}
