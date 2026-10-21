use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub const DESCRIPTION: &str = "HTML entity encode / decode";

pub struct Feature;

fn encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_language: Some("html"),
            output_language: Some("html"),
            ..Default::default()
        });
        tool.add_button("Encode", |t| t.report("Encoded", Ok(encode(&t.input()))));
        tool.add_button_with_class("Decode", None, |t| {
            t.report(
                "Decoded",
                Ok(html_escape::decode_html_entities(&t.input()).into_owned()),
            )
        });
        tool.into_widget()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn escapes_like_python_html_escape() {
        assert_eq!(
            super::encode("<a href=\"x\">'&'</a>"),
            "&lt;a href=&quot;x&quot;&gt;&#x27;&amp;&#x27;&lt;/a&gt;"
        );
    }
}
