use chrono::{Local, Utc};
use gtk::prelude::*;

use crate::domain::feature::parsing::cron_parser::{self, EXAMPLES};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

const NEXT_RUNS: usize = 10;

pub struct Feature;

fn explain(t: &InputOutputWidget) {
    if t.input().trim().is_empty() {
        t.set_output("");
    } else {
        t.report(cron_parser::explain(
            &t.input(),
            Utc::now(),
            &Local,
            NEXT_RUNS,
        ));
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Cron expression",
            output_label: "Schedule",
            ..Default::default()
        });

        let mut labels = vec!["Examples…".to_string()];
        labels.extend(
            EXAMPLES
                .iter()
                .map(|(expr, meaning)| format!("{expr}   {meaning}")),
        );
        let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
        let presets = gtk::DropDown::from_strings(&labels);
        presets.set_tooltip_text(Some("Common schedules"));
        let weak_tool = tool.downgrade();
        presets.connect_selected_notify(move |dropdown| {
            let preset = (dropdown.selected() as usize)
                .checked_sub(1)
                .and_then(|i| EXAMPLES.get(i));
            if let (Some((expr, _)), Some(tool)) = (preset, weak_tool.upgrade()) {
                tool.set_input(expr);
            }
        });
        tool.add_extra_widget(&presets);

        tool.connect_input_changed(explain);
        tool.set_input(EXAMPLES[0].0);
        tool.into_widget()
    }
}
