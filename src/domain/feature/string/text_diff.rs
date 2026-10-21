use similar::{ChangeTag, TextDiff};

use crate::domain::entity::feature_entity::FeatureEntity;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "text_diff",
    section: "string",
    title: "Text Diff",
    description: "Compare two texts line by line",
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiffOptions {
    /// Compare lines without leading/trailing whitespace.
    pub ignore_whitespace: bool,
    pub ignore_case: bool,
    /// Unchanged lines kept around each change in the unified output.
    pub context_lines: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
}

impl DiffStats {
    pub fn is_identical(&self) -> bool {
        self.added == 0 && self.removed == 0
    }

    pub fn summary(&self) -> String {
        if self.is_identical() {
            "No differences".into()
        } else {
            format!(
                "+{} added, −{} removed, {} unchanged",
                self.added, self.removed, self.unchanged
            )
        }
    }
}

/// Applies the ignore options line by line. Every line ends with `\n`, so
/// a missing final newline does not count as a change of the last line.
fn normalize(text: &str, options: DiffOptions) -> String {
    text.lines()
        .map(|line| {
            let line = if options.ignore_whitespace {
                line.trim()
            } else {
                line
            };
            let line = if options.ignore_case {
                line.to_lowercase()
            } else {
                line.to_string()
            };
            line + "\n"
        })
        .collect()
}

/// Unified diff (`--- original` / `+++ modified`) and line statistics.
/// With ignore options, the output shows the normalized lines.
pub fn diff(original: &str, modified: &str, options: DiffOptions) -> (String, DiffStats) {
    let (old, new) = (normalize(original, options), normalize(modified, options));
    let text_diff = TextDiff::from_lines(&old, &new);

    let mut stats = DiffStats::default();
    for change in text_diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => stats.added += 1,
            ChangeTag::Delete => stats.removed += 1,
            ChangeTag::Equal => stats.unchanged += 1,
        }
    }

    let unified = text_diff
        .unified_diff()
        .context_radius(options.context_lines)
        .header("original", "modified")
        .to_string();
    (unified, stats)
}
