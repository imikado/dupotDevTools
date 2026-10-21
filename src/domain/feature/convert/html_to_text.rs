use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::DomainResult;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "html_to_text",
    section: "convert",
    title: "HTML to Text",
    description: "Strip HTML tags to plain text",
};

const LINE_WIDTH: usize = 80;

/// Renders HTML as readable plain text (links kept as footnotes).
pub fn convert(html: &str) -> DomainResult<String> {
    Ok(html2text::from_read(html.as_bytes(), LINE_WIDTH)?)
}
