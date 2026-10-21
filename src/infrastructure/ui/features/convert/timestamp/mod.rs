use chrono::{Local, Utc};

use crate::domain::feature::convert::timestamp;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

fn convert(t: &InputOutputWidget) {
    if t.input().trim().is_empty() {
        t.set_output("");
    } else {
        t.report(timestamp::describe(&t.input(), Utc::now(), &Local));
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Timestamp or date",
            output_label: "Conversions",
            ..Default::default()
        });
        tool.add_button("Now", |t| t.set_input(&Utc::now().timestamp().to_string()));
        tool.add_button_with_class("Now (ms)", None, |t| {
            t.set_input(&Utc::now().timestamp_millis().to_string())
        });
        tool.connect_input_changed(convert);
        tool.into_widget()
    }
}
