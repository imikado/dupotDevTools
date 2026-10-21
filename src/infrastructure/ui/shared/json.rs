use serde::Serialize;

/// JSON with 4-space indentation and raw (non-escaped) unicode.
pub fn to_pretty_json(value: &impl Serialize) -> Result<String, String> {
    let mut out = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut out, formatter);
    value.serialize(&mut ser).map_err(|e| e.to_string())?;
    String::from_utf8(out).map_err(|e| e.to_string())
}
