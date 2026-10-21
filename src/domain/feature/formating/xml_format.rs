use quick_xml::events::{BytesDecl, Event};
use quick_xml::{Reader, Writer};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "xml_format",
    section: "formating",
    title: "XML Formatter",
    description: "Format XML documents",
};

/// Re-indents an XML document with 4 spaces, adding an XML declaration when
/// missing. Fails on malformed or empty documents.
pub fn format(input: &str) -> DomainResult<String> {
    let mut reader = Reader::from_str(input.trim());
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 4);
    let mut depth = 0usize;
    let mut has_root = false;
    let mut first = true;
    loop {
        let event = reader
            .read_event()
            .map_err(|e| DomainError::new(format!("{e} (at byte {})", reader.error_position())))?;
        if first {
            first = false;
            if !matches!(event, Event::Decl(_)) {
                writer.write_event(Event::Decl(BytesDecl::new("1.0", None, None)))?;
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
        writer.write_event(event)?;
    }
    if !has_root {
        return Err(DomainError::new("no element found"));
    }
    if depth != 0 {
        return Err(DomainError::new("unclosed element at end of document"));
    }
    Ok(String::from_utf8(writer.into_inner())?)
}
