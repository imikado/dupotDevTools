use gtk::prelude::*;

use crate::domain::feature::string::hash::{HashAlgorithm, hash};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            output_label: "Hash",
            output_height: 120,
            ..Default::default()
        });

        let labels: Vec<&str> = HashAlgorithm::ALL.iter().map(|a| a.label()).collect();
        let dropdown = gtk::DropDown::from_strings(&labels);
        let default_index = HashAlgorithm::ALL
            .iter()
            .position(|a| *a == HashAlgorithm::DEFAULT)
            .unwrap_or(0);
        dropdown.set_selected(default_index as u32);
        dropdown.set_tooltip_text(Some("Algorithm"));
        let algo_lbl = gtk::Label::new(Some("Algorithm"));
        algo_lbl.add_css_class("dim-label");
        tool.add_extra_widget(&algo_lbl);
        tool.add_extra_widget(&dropdown);

        tool.add_button("Hash", move |t| {
            let algorithm = HashAlgorithm::ALL[dropdown.selected() as usize];
            t.report(Ok(hash(algorithm, t.input().as_bytes())));
        });
        tool.into_widget()
    }
}
