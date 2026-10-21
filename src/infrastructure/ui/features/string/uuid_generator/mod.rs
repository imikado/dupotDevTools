use chrono::Local;
use gtk::prelude::*;

use crate::domain::feature::string::uuid_generator::{
    IdFormat, IdKind, MAX_COUNT, generate, inspect,
};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Inspect a UUID or ULID",
            output_label: "Result",
            ..Default::default()
        });

        let labels: Vec<&str> = IdKind::ALL.iter().map(|k| k.label()).collect();
        let kind = gtk::DropDown::from_strings(&labels);
        let count = gtk::SpinButton::with_range(1.0, MAX_COUNT as f64, 1.0);
        count.set_tooltip_text(Some("How many"));
        let uppercase = gtk::CheckButton::with_label("Uppercase");
        let no_hyphens = gtk::CheckButton::with_label("No hyphens");
        let braces = gtk::CheckButton::with_label("Braces");
        tool.add_extra_widget(&kind);
        tool.add_extra_widget(&count);
        tool.add_extra_widget(&uppercase);
        tool.add_extra_widget(&no_hyphens);
        tool.add_extra_widget(&braces);

        tool.add_button("Generate", move |t| {
            let format = IdFormat {
                uppercase: uppercase.is_active(),
                no_hyphens: no_hyphens.is_active(),
                braces: braces.is_active(),
            };
            let kind = IdKind::ALL[kind.selected() as usize];
            t.set_output(&generate(kind, count.value() as usize, format));
        });
        tool.connect_input_changed(|t| {
            if t.input().trim().is_empty() {
                t.set_output("");
            } else {
                t.report(inspect(&t.input(), &Local));
            }
        });
        tool.into_widget()
    }
}
