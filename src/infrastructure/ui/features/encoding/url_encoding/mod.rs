use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};

use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub const DESCRIPTION: &str = "URL percent-encode / decode";

/// RFC 3986 unreserved characters stay as-is, everything else is encoded.
const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options::default());
        tool.add_button("Encode", |t| {
            t.report(
                "Encoded",
                Ok(utf8_percent_encode(&t.input(), ENCODE_SET).to_string()),
            )
        });
        tool.add_button_with_class("Decode", None, |t| {
            t.report(
                "Decoded",
                Ok(percent_decode_str(&t.input())
                    .decode_utf8_lossy()
                    .into_owned()),
            )
        });
        tool.into_widget()
    }
}
