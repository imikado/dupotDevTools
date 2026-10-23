use gtk::prelude::*;

use crate::domain::feature::parsing::xpath_tester::{SAMPLE, evaluate};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

fn update(t: &InputOutputWidget, expression: &gtk::Entry) {
    if expression.text().trim().is_empty() || t.input().trim().is_empty() {
        t.set_output("");
    } else {
        t.report(evaluate(&expression.text(), &t.input()));
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "XML",
            output_label: "Result",
            input_language: Some("xml"),
            output_language: Some("xml"),
            input_height: 260,
            output_height: 260,
        });

        let expression = gtk::Entry::builder()
            .placeholder_text("XPath, e.g. //book[@lang='en']/title")
            .hexpand(true)
            .build();
        expression.add_css_class("monospace");
        tool.add_extra_widget(&expression);

        let weak_tool = tool.downgrade();
        expression.connect_changed(move |entry| {
            if let Some(t) = weak_tool.upgrade() {
                update(&t, entry);
            }
        });
        let e = expression.clone();
        tool.connect_input_changed(move |t| update(t, &e));
        tool.set_input(SAMPLE);
        expression.set_text("//book[price < 9]/title");
        tool.into_widget()
    }
}
