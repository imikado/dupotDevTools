use gtk::prelude::*;

use crate::domain::feature::string::case_converter::{Case, convert_all, convert_lines};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

fn update(t: &InputOutputWidget, target: &gtk::DropDown) {
    let text = t.input();
    let output = match (target.selected() as usize).checked_sub(1) {
        Some(i) => convert_lines(&text, Case::ALL[i]),
        None => convert_all(&text),
    };
    t.set_output(&output);
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Text (one identifier or phrase per line)",
            output_label: "Converted",
            input_height: 260,
            output_height: 260,
            ..Default::default()
        });

        let labels: Vec<&str> = std::iter::once("All cases")
            .chain(Case::ALL.iter().map(|c| c.label()))
            .collect();
        let target = gtk::DropDown::from_strings(&labels);
        target.set_tooltip_text(Some("Target case"));
        tool.add_extra_widget(&target);

        let weak_tool = tool.downgrade();
        target.connect_selected_notify(move |target| {
            if let Some(t) = weak_tool.upgrade() {
                update(&t, target);
            }
        });
        tool.connect_input_changed(move |t| update(t, &target));
        tool.set_input("user_account_id\nHTTPServerError\nmy-component name");
        tool.into_widget()
    }
}
