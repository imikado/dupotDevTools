use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::shared::format::rows;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "case_converter",
    section: "string",
    title: "Case Converter",
    description: "Convert text between camelCase, PascalCase, snake_case, kebab-case and more",
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Camel,
    Pascal,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
    Train,
    Dot,
    Path,
    Title,
    Sentence,
    Lower,
    Upper,
}

impl Case {
    pub const ALL: [Case; 13] = [
        Case::Camel,
        Case::Pascal,
        Case::Snake,
        Case::ScreamingSnake,
        Case::Kebab,
        Case::ScreamingKebab,
        Case::Train,
        Case::Dot,
        Case::Path,
        Case::Title,
        Case::Sentence,
        Case::Lower,
        Case::Upper,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Case::Camel => "camelCase",
            Case::Pascal => "PascalCase",
            Case::Snake => "snake_case",
            Case::ScreamingSnake => "SCREAMING_SNAKE_CASE",
            Case::Kebab => "kebab-case",
            Case::ScreamingKebab => "COBOL-CASE",
            Case::Train => "Train-Case",
            Case::Dot => "dot.case",
            Case::Path => "path/case",
            Case::Title => "Title Case",
            Case::Sentence => "Sentence case",
            Case::Lower => "lower case",
            Case::Upper => "UPPER CASE",
        }
    }
}

/// Splits an identifier or phrase into words: on any non-alphanumeric
/// character, before an upper-case letter following a lower-case one or a
/// digit (`fooBar`, `utf8Decode`), and before the last capital of an acronym
/// (`HTTPServer` → `HTTP`, `Server`).
pub fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    for chunk in text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|c| !c.is_empty())
    {
        let chars: Vec<char> = chunk.chars().collect();
        let mut word = String::new();
        for (i, &c) in chars.iter().enumerate() {
            if i > 0 && c.is_uppercase() {
                let prev = chars[i - 1];
                let next_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
                if prev.is_lowercase() || prev.is_numeric() || (prev.is_uppercase() && next_lower) {
                    words.push(std::mem::take(&mut word));
                }
            }
            word.push(c);
        }
        words.push(word);
    }
    words
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first
            .to_uppercase()
            .chain(chars.flat_map(char::to_lowercase))
            .collect(),
        None => String::new(),
    }
}

pub fn convert(text: &str, case: Case) -> String {
    let words = words(text);
    let lower: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let upper: Vec<String> = words.iter().map(|w| w.to_uppercase()).collect();
    let capitalized: Vec<String> = words.iter().map(|w| capitalize(w)).collect();
    match case {
        Case::Camel => lower
            .first()
            .into_iter()
            .cloned()
            .chain(capitalized.iter().skip(1).cloned())
            .collect(),
        Case::Pascal => capitalized.concat(),
        Case::Snake => lower.join("_"),
        Case::ScreamingSnake => upper.join("_"),
        Case::Kebab => lower.join("-"),
        Case::ScreamingKebab => upper.join("-"),
        Case::Train => capitalized.join("-"),
        Case::Dot => lower.join("."),
        Case::Path => lower.join("/"),
        Case::Title => capitalized.join(" "),
        Case::Sentence => capitalize(&lower.join(" ")),
        Case::Lower => lower.join(" "),
        Case::Upper => upper.join(" "),
    }
}

/// Each line converted to `case`.
pub fn convert_lines(text: &str, case: Case) -> String {
    text.lines()
        .map(|line| convert(line, case))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every case of every non-empty line, one block per line.
pub fn convert_all(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let table: Vec<(&str, String)> = Case::ALL
                .iter()
                .map(|case| (case.label(), convert(line, *case)))
                .collect();
            rows(&table)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}
