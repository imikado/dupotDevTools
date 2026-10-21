use sqlformat::{FormatOptions, Indent, QueryParams};

use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub const TITLE: &str = "SQL Formatter";
pub const DESCRIPTION: &str = "Format SQL queries";

pub struct Feature;

fn format(input: &str) -> String {
    let options = FormatOptions {
        indent: Indent::Spaces(4),
        uppercase: Some(true),
        ..Default::default()
    };
    sqlformat::format(input, &QueryParams::None, &options)
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input SQL",
            input_language: Some("sql"),
            output_language: Some("sql"),
            ..Default::default()
        });
        tool.add_button("Format", |t| t.report(Ok(format(&t.input()))));
        tool.into_widget()
    }
}
