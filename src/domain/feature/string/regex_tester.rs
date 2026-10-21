use regex::{Regex, RegexBuilder};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "regex_tester",
    section: "string",
    title: "Regex Tester",
    description: "Test regular expressions with live highlighting",
};

/// Matches beyond this count are ignored (keeps highlighting responsive).
pub const MAX_MATCHES: usize = 1000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegexOptions {
    pub case_insensitive: bool,
    /// `^` and `$` match at line boundaries.
    pub multiline: bool,
    /// `.` also matches `\n`.
    pub dot_matches_newline: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// Group number (1-based) or name.
    pub label: String,
    /// `None` when the group did not participate in the match.
    pub value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    /// Byte offsets in the searched text.
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub groups: Vec<Group>,
}

pub fn compile(pattern: &str, options: RegexOptions) -> DomainResult<Regex> {
    if pattern.is_empty() {
        return Err(DomainError::new("Enter a regular expression."));
    }
    Ok(RegexBuilder::new(pattern)
        .case_insensitive(options.case_insensitive)
        .multi_line(options.multiline)
        .dot_matches_new_line(options.dot_matches_newline)
        .build()?)
}

/// Every match of `pattern` in `text` (at most `MAX_MATCHES`), with groups.
pub fn find_matches(pattern: &str, options: RegexOptions, text: &str) -> DomainResult<Vec<Match>> {
    let regex = compile(pattern, options)?;
    let names: Vec<Option<&str>> = regex.capture_names().collect();
    Ok(regex
        .captures_iter(text)
        .take(MAX_MATCHES)
        .map(|caps| {
            let whole = caps.get(0).expect("group 0 always matches");
            let groups = (1..caps.len())
                .map(|i| Group {
                    label: names[i].map_or_else(|| i.to_string(), str::to_string),
                    value: caps.get(i).map(|m| m.as_str().to_string()),
                })
                .collect();
            Match {
                start: whole.start(),
                end: whole.end(),
                text: whole.as_str().to_string(),
                groups,
            }
        })
        .collect())
}

/// Human-readable list of matches and their groups.
pub fn report(matches: &[Match]) -> String {
    if matches.is_empty() {
        return "No match".into();
    }
    let mut out = match matches.len() {
        1 => "1 match".to_string(),
        n if n >= MAX_MATCHES => format!("{n}+ matches (showing the first {MAX_MATCHES})"),
        n => format!("{n} matches"),
    };
    for (i, m) in matches.iter().enumerate() {
        out.push_str(&format!(
            "\n\n#{}  [{}..{}]  {:?}",
            i + 1,
            m.start,
            m.end,
            m.text
        ));
        for g in &m.groups {
            match &g.value {
                Some(v) => out.push_str(&format!("\n    {}: {v:?}", g.label)),
                None => out.push_str(&format!("\n    {}: —", g.label)),
            }
        }
    }
    out
}

/// Replaces every match; `$1`, `${name}` refer to groups.
pub fn replace(
    pattern: &str,
    options: RegexOptions,
    text: &str,
    replacement: &str,
) -> DomainResult<String> {
    Ok(compile(pattern, options)?
        .replace_all(text, replacement)
        .into_owned())
}
