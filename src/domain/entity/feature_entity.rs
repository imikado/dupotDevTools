/// What a tool is, independently of how it is displayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeatureEntity {
    /// Identifier, also the tool's folder name (`base64_encoding`).
    pub name: &'static str,
    /// Section (sidebar category) identifier (`encoding`).
    pub section: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

impl FeatureEntity {
    /// Case-insensitive match of `query` (already lowercased) against the
    /// title, description, name and section.
    pub fn matches(&self, query: &str) -> bool {
        [self.title, self.description, self.name, self.section]
            .iter()
            .any(|s| s.to_lowercase().contains(query))
    }
}
