use quick_xml::events::{BytesDecl, Event};
use quick_xml::{Reader, Writer};

use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub const DESCRIPTION: &str = "Format XML documents";

pub struct Feature;

fn format(input: &str) -> Result<String, String> {
    let mut reader = Reader::from_str(input.trim());
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 4);
    let mut depth = 0usize;
    let mut has_root = false;
    let mut first = true;
    loop {
        let event = reader
            .read_event()
            .map_err(|e| format!("{e} (at byte {})", reader.error_position()))?;
        if first {
            first = false;
            if !matches!(event, Event::Decl(_)) {
                writer
                    .write_event(Event::Decl(BytesDecl::new("1.0", None, None)))
                    .map_err(|e| e.to_string())?;
            }
        }
        match &event {
            Event::Eof => break,
            Event::Start(_) => {
                depth += 1;
                has_root = true;
            }
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Empty(_) => has_root = true,
            _ => {}
        }
        writer.write_event(event).map_err(|e| e.to_string())?;
    }
    if !has_root {
        return Err("no element found".into());
    }
    if depth != 0 {
        return Err("unclosed element at end of document".into());
    }
    String::from_utf8(writer.into_inner()).map_err(|e| e.to_string())
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Input XML",
            input_language: Some("xml"),
            output_language: Some("xml"),
            ..Default::default()
        });
        tool.add_button("Format", |t| t.report("Formatted", format(&t.input())));
        tool.into_widget()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn indents_and_rejects_bad_xml() {
        let out = super::format("<a><b>x</b><c/></a>").unwrap();
        assert_eq!(
            out,
            "<?xml version=\"1.0\"?>\n<a>\n    <b>x</b>\n    <c/>\n</a>"
        );
        assert!(super::format("<a><b></a>").is_err());
        assert!(super::format("").is_err());
    }
}
