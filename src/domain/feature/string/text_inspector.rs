use std::collections::BTreeMap;

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::shared::format::rows;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "text_inspector",
    section: "string",
    title: "Text Inspector",
    description: "Count characters, words and lines; reveal invisible and special Unicode characters",
};

/// Listed special characters beyond this count are only counted.
pub const MAX_LISTED: usize = 200;

const C0_NAMES: [&str; 32] = [
    "NULL",
    "START OF HEADING",
    "START OF TEXT",
    "END OF TEXT",
    "END OF TRANSMISSION",
    "ENQUIRY",
    "ACKNOWLEDGE",
    "BELL",
    "BACKSPACE",
    "CHARACTER TABULATION",
    "LINE FEED",
    "LINE TABULATION",
    "FORM FEED",
    "CARRIAGE RETURN",
    "SHIFT OUT",
    "SHIFT IN",
    "DATA LINK ESCAPE",
    "DEVICE CONTROL ONE",
    "DEVICE CONTROL TWO",
    "DEVICE CONTROL THREE",
    "DEVICE CONTROL FOUR",
    "NEGATIVE ACKNOWLEDGE",
    "SYNCHRONOUS IDLE",
    "END OF TRANSMISSION BLOCK",
    "CANCEL",
    "END OF MEDIUM",
    "SUBSTITUTE",
    "ESCAPE",
    "INFORMATION SEPARATOR FOUR",
    "INFORMATION SEPARATOR THREE",
    "INFORMATION SEPARATOR TWO",
    "INFORMATION SEPARATOR ONE",
];

/// Name of an invisible, confusable or otherwise surprising character, or
/// `None` for ordinary ones (tabs and line breaks included).
pub fn special_name(c: char) -> Option<String> {
    let name = match c {
        '\t' | '\n' | '\r' => return None,
        '\u{0}'..='\u{1F}' => C0_NAMES[c as usize],
        '\u{7F}' => "DELETE",
        '\u{80}'..='\u{9F}' => "C1 CONTROL",
        '\u{A0}' => "NO-BREAK SPACE",
        '\u{AD}' => "SOFT HYPHEN",
        '\u{34F}' => "COMBINING GRAPHEME JOINER",
        '\u{61C}' => "ARABIC LETTER MARK",
        '\u{115F}' | '\u{1160}' | '\u{3164}' | '\u{FFA0}' => "HANGUL FILLER",
        '\u{180E}' => "MONGOLIAN VOWEL SEPARATOR",
        '\u{2000}' => "EN QUAD",
        '\u{2001}' => "EM QUAD",
        '\u{2002}' => "EN SPACE",
        '\u{2003}' => "EM SPACE",
        '\u{2004}' => "THREE-PER-EM SPACE",
        '\u{2005}' => "FOUR-PER-EM SPACE",
        '\u{2006}' => "SIX-PER-EM SPACE",
        '\u{2007}' => "FIGURE SPACE",
        '\u{2008}' => "PUNCTUATION SPACE",
        '\u{2009}' => "THIN SPACE",
        '\u{200A}' => "HAIR SPACE",
        '\u{200B}' => "ZERO WIDTH SPACE",
        '\u{200C}' => "ZERO WIDTH NON-JOINER",
        '\u{200D}' => "ZERO WIDTH JOINER",
        '\u{200E}' => "LEFT-TO-RIGHT MARK",
        '\u{200F}' => "RIGHT-TO-LEFT MARK",
        '\u{2028}' => "LINE SEPARATOR",
        '\u{2029}' => "PARAGRAPH SEPARATOR",
        '\u{202A}' => "LEFT-TO-RIGHT EMBEDDING",
        '\u{202B}' => "RIGHT-TO-LEFT EMBEDDING",
        '\u{202C}' => "POP DIRECTIONAL FORMATTING",
        '\u{202D}' => "LEFT-TO-RIGHT OVERRIDE",
        '\u{202E}' => "RIGHT-TO-LEFT OVERRIDE",
        '\u{202F}' => "NARROW NO-BREAK SPACE",
        '\u{205F}' => "MEDIUM MATHEMATICAL SPACE",
        '\u{2060}' => "WORD JOINER",
        '\u{2061}' => "FUNCTION APPLICATION",
        '\u{2062}' => "INVISIBLE TIMES",
        '\u{2063}' => "INVISIBLE SEPARATOR",
        '\u{2064}' => "INVISIBLE PLUS",
        '\u{2066}' => "LEFT-TO-RIGHT ISOLATE",
        '\u{2067}' => "RIGHT-TO-LEFT ISOLATE",
        '\u{2068}' => "FIRST STRONG ISOLATE",
        '\u{2069}' => "POP DIRECTIONAL ISOLATE",
        '\u{206A}'..='\u{206F}' => "DEPRECATED FORMAT CHARACTER",
        '\u{3000}' => "IDEOGRAPHIC SPACE",
        '\u{FE00}'..='\u{FE0F}' => {
            return Some(format!("VARIATION SELECTOR-{}", c as u32 - 0xFE00 + 1));
        }
        '\u{FEFF}' => "ZERO WIDTH NO-BREAK SPACE (BOM)",
        '\u{FFF9}'..='\u{FFFB}' => "INTERLINEAR ANNOTATION",
        '\u{FFFC}' => "OBJECT REPLACEMENT CHARACTER",
        '\u{FFFD}' => "REPLACEMENT CHARACTER (decoding error?)",
        '\u{E0001}' => "LANGUAGE TAG",
        '\u{E0020}'..='\u{E007F}' => "TAG CHARACTER",
        '\u{E0100}'..='\u{E01EF}' => "VARIATION SELECTOR SUPPLEMENT",
        '\u{E000}'..='\u{F8FF}' | '\u{F0000}'..='\u{10FFFF}' => "PRIVATE USE",
        _ => return None,
    };
    Some(name.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialChar {
    pub line: usize,
    /// 1-based, in characters.
    pub column: usize,
    /// Byte range in the text.
    pub start: usize,
    pub end: usize,
    pub ch: char,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inspection {
    pub report: String,
    pub specials: Vec<SpecialChar>,
}

fn line_endings(text: &str) -> String {
    let crlf = text.matches("\r\n").count();
    let cr = text.matches('\r').count() - crlf;
    let lf = text.matches('\n').count() - crlf;
    let kinds: Vec<String> = [(lf, "LF"), (crlf, "CRLF"), (cr, "CR")]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, name)| format!("{n} {name}"))
        .collect();
    match kinds.len() {
        0 => "none".into(),
        1 => kinds[0].clone(),
        _ => format!("mixed: {}", kinds.join(", ")),
    }
}

fn specials(text: &str) -> Vec<SpecialChar> {
    let mut out = Vec::new();
    let mut line_start = 0;
    for (line_index, line) in text.split('\n').enumerate() {
        for (column, (i, ch)) in line.char_indices().enumerate() {
            if let Some(name) = special_name(ch) {
                let start = line_start + i;
                out.push(SpecialChar {
                    line: line_index + 1,
                    column: column + 1,
                    start,
                    end: start + ch.len_utf8(),
                    ch,
                    name,
                });
            }
        }
        line_start += line.len() + 1;
    }
    out
}

/// Counters, line ending style, special characters (with positions, for
/// highlighting) and the non-ASCII characters used.
pub fn inspect(text: &str) -> Inspection {
    let chars = text.chars().count();
    let lines: Vec<&str> = text.lines().collect();
    let trailing: Vec<String> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.ends_with([' ', '\t']))
        .map(|(i, _)| (i + 1).to_string())
        .collect();
    let paragraphs = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    let mut non_ascii: BTreeMap<char, usize> = BTreeMap::new();
    for c in text.chars().filter(|c| !c.is_ascii()) {
        *non_ascii.entry(c).or_default() += 1;
    }

    let mut summary = vec![
        ("Characters", chars.to_string()),
        (
            "Bytes",
            format!(
                "{} UTF-8 · {} UTF-16",
                text.len(),
                text.encode_utf16().count() * 2
            ),
        ),
        ("Words", text.split_whitespace().count().to_string()),
        ("Lines", lines.len().to_string()),
        ("Line endings", line_endings(text)),
        ("Paragraphs", paragraphs.to_string()),
        (
            "Longest line",
            format!(
                "{} characters",
                lines.iter().map(|l| l.chars().count()).max().unwrap_or(0)
            ),
        ),
        (
            "Whitespace",
            text.chars()
                .filter(|c| c.is_whitespace())
                .count()
                .to_string(),
        ),
        ("Tabs", text.matches('\t').count().to_string()),
        ("Non-ASCII", non_ascii.values().sum::<usize>().to_string()),
    ];
    if !trailing.is_empty() {
        let mut listed = trailing
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if trailing.len() > 20 {
            listed.push_str(", …");
        }
        summary.push((
            "Trailing spaces",
            format!(
                "{} line{} ({listed})",
                trailing.len(),
                if trailing.len() == 1 { "" } else { "s" }
            ),
        ));
    }
    if text.starts_with('\u{FEFF}') {
        summary.push(("BOM", "yes (UTF-8 byte order mark at start)".into()));
    }

    let specials = specials(text);
    let mut report = rows(&summary);
    if !specials.is_empty() {
        report.push_str(&format!(
            "\n\nSpecial characters ({}, highlighted)\n",
            specials.len()
        ));
        let listed: Vec<(String, String)> = specials
            .iter()
            .take(MAX_LISTED)
            .map(|s| {
                (
                    format!("{}:{}", s.line, s.column),
                    format!("U+{:04X}  {}", s.ch as u32, s.name),
                )
            })
            .collect();
        report.push_str(&rows(&listed));
        if specials.len() > MAX_LISTED {
            report.push_str(&format!("\n… and {} more", specials.len() - MAX_LISTED));
        }
    }
    let visible: Vec<(String, String)> = non_ascii
        .iter()
        .filter(|(c, _)| special_name(**c).is_none())
        .take(MAX_LISTED)
        .map(|(c, n)| (format!("{c}"), format!("U+{:04X}  ×{n}", *c as u32)))
        .collect();
    if !visible.is_empty() {
        report.push_str("\n\nNon-ASCII characters\n");
        report.push_str(&rows(&visible));
    }
    Inspection { report, specials }
}
