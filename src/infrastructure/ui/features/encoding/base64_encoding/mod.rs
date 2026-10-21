use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub const DESCRIPTION: &str = "Encode / decode Base64";

pub struct Feature;

fn decode(input: &str) -> Result<String, String> {
    let bytes = STANDARD.decode(input.trim()).map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options::default());
        tool.add_button("Encode", |t| {
            t.report("Encoded", Ok(STANDARD.encode(t.input())))
        });
        tool.add_button_with_class("Decode", None, |t| t.report("Decoded", decode(&t.input())));
        tool.into_widget()
    }
}
