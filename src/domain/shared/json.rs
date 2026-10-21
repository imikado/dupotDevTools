use serde::Serialize;

use crate::domain::error::DomainResult;

/// JSON with 4-space indentation and raw (non-escaped) unicode.
pub fn to_pretty_json(value: &impl Serialize) -> DomainResult<String> {
    let mut out = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut out, formatter);
    value.serialize(&mut ser)?;
    Ok(String::from_utf8(out)?)
}
