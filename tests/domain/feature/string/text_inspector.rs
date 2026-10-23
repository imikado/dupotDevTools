use dupot_dev_tools::domain::feature::string::text_inspector::{inspect, special_name};

/// The value of the report row starting with `label`.
fn row(report: &str, label: &str) -> String {
    report
        .lines()
        .find_map(|l| l.strip_prefix(label))
        .unwrap_or_else(|| panic!("no {label} in {report}"))
        .trim()
        .to_string()
}

#[test]
fn counts_text() {
    let report = inspect("Hello world\r\nsecond line \n\nnew paragraph").report;
    assert_eq!(row(&report, "Characters"), "40");
    assert_eq!(row(&report, "Words"), "6");
    assert_eq!(row(&report, "Lines"), "4");
    assert_eq!(row(&report, "Line endings"), "mixed: 2 LF, 1 CRLF");
    assert_eq!(row(&report, "Paragraphs"), "2");
    assert_eq!(row(&report, "Trailing spaces"), "1 line (2)");
}

#[test]
fn counts_bytes_in_utf8_and_utf16() {
    let report = inspect("é😀").report;
    assert_eq!(row(&report, "Characters"), "2");
    assert_eq!(row(&report, "Bytes"), "6 UTF-8 · 6 UTF-16");
    assert!(report.contains("U+1F600  ×1"), "{report}");
}

#[test]
fn locates_special_characters() {
    let text = "ok\na\u{200B}b\u{A0}";
    let inspection = inspect(text);
    let specials = &inspection.specials;
    assert_eq!(specials.len(), 2);
    assert_eq!((specials[0].line, specials[0].column), (2, 2));
    assert_eq!(&text[specials[0].start..specials[0].end], "\u{200B}");
    assert_eq!(specials[1].name, "NO-BREAK SPACE");
    assert!(
        inspection.report.contains("2:2  U+200B  ZERO WIDTH SPACE"),
        "{}",
        inspection.report
    );
    // Specials are not repeated in the non-ASCII list.
    assert!(!inspection.report.contains("Non-ASCII characters"));
}

#[test]
fn names_special_characters() {
    assert_eq!(special_name('a'), None);
    assert_eq!(special_name('\t'), None);
    assert_eq!(special_name('\u{0}').as_deref(), Some("NULL"));
    assert_eq!(
        special_name('\u{202E}').as_deref(),
        Some("RIGHT-TO-LEFT OVERRIDE")
    );
    assert_eq!(
        special_name('\u{FE0F}').as_deref(),
        Some("VARIATION SELECTOR-16")
    );
}

#[test]
fn flags_a_bom() {
    assert!(inspect("\u{FEFF}x").report.contains("BOM"));
}
