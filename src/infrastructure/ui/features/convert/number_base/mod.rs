use crate::domain::feature::convert::number_base::{self, EXAMPLES};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

fn convert(t: &InputOutputWidget) {
    if t.input().trim().is_empty() {
        t.set_output("");
    } else {
        t.report(number_base::describe(&t.input()));
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Number or expression (0x, 0o, 0b prefixes; | & ^ ~ << >>)",
            output_label: "Bases",
            ..Default::default()
        });
        tool.add_examples(&EXAMPLES);
        tool.connect_input_changed(convert);
        tool.set_input(EXAMPLES[0]);
        tool.into_widget()
    }
}
