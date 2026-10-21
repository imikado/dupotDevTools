use crate::domain::entity::feature_entity::FeatureEntity;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "html_encoding",
    section: "encoding",
    title: "HTML Entities",
    description: "HTML entity encode / decode",
};

/// Escapes `& < > " '`, like Python's `html.escape`.
pub fn encode(input: &str) -> String {
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

/// Decodes named and numeric HTML entities.
pub fn decode(input: &str) -> String {
    html_escape::decode_html_entities(input).into_owned()
}
