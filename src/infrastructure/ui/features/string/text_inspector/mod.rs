use crate::domain::feature::string::text_inspector::inspect;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

fn update(t: &InputOutputWidget) {
    let text = t.input();
    if text.is_empty() {
        t.highlight_input(&[]);
        t.set_output("");
        return;
    }
    let inspection = inspect(&text);
    let ranges: Vec<_> = inspection
        .specials
        .iter()
        .map(|s| (s.start, s.end))
        .collect();
    t.highlight_input(&ranges);
    t.set_output(&inspection.report);
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Text",
            output_label: "Report",
            input_height: 260,
            output_height: 260,
            ..Default::default()
        });
        tool.connect_input_changed(update);
        tool.into_widget()
    }
}
