use crate::domain::feature::convert::unit_converter::{self, EXAMPLES};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

fn convert(t: &InputOutputWidget) {
    if t.input().trim().is_empty() {
        t.set_output("");
    } else {
        t.report(unit_converter::describe(&t.input()));
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Size or duration",
            output_label: "Conversions",
            ..Default::default()
        });
        tool.add_examples(&EXAMPLES);
        tool.connect_input_changed(convert);
        tool.set_input(EXAMPLES[0]);
        tool.into_widget()
    }
}
