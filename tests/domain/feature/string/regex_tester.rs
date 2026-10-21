use dupot_dev_tools::domain::feature::string::regex_tester::{
    RegexOptions, find_matches, replace, report,
};

#[test]
fn finds_matches_with_byte_offsets_and_groups() {
    let matches = find_matches(
        r"(?P<user>\w+)@(\w+)\.com",
        RegexOptions::default(),
        "a: bob@ex.com, é ann@b.com",
    )
    .unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!((matches[0].start, matches[0].end), (3, 13));
    assert_eq!(matches[0].text, "bob@ex.com");
    assert_eq!(matches[0].groups[0].label, "user");
    assert_eq!(matches[0].groups[0].value.as_deref(), Some("bob"));
    assert_eq!(matches[0].groups[1].label, "2");
    // "é" is 2 bytes: offsets are in bytes.
    assert_eq!(matches[1].start, 18);
}

#[test]
fn options_change_matching() {
    let text = "Hello\nhello";
    assert_eq!(
        find_matches("^hello$", RegexOptions::default(), text)
            .unwrap()
            .len(),
        0
    );
    let ci_ml = RegexOptions {
        case_insensitive: true,
        multiline: true,
        ..Default::default()
    };
    assert_eq!(find_matches("^hello$", ci_ml, text).unwrap().len(), 2);
    let dot_all = RegexOptions {
        dot_matches_newline: true,
        ..Default::default()
    };
    assert_eq!(find_matches("o.h", dot_all, text).unwrap().len(), 1);
}

#[test]
fn invalid_and_empty_patterns_are_errors() {
    assert!(find_matches("(", RegexOptions::default(), "x").is_err());
    assert_eq!(
        find_matches("", RegexOptions::default(), "x")
            .unwrap_err()
            .message(),
        "Enter a regular expression."
    );
}

#[test]
fn report_lists_matches_and_unmatched_groups() {
    let matches = find_matches(r"a(b)?", RegexOptions::default(), "ab a").unwrap();
    assert_eq!(
        report(&matches),
        "2 matches\n\n#1  [0..2]  \"ab\"\n    1: \"b\"\n\n#2  [3..4]  \"a\"\n    1: —"
    );
    assert_eq!(report(&[]), "No match");
}

#[test]
fn replace_with_group_references() {
    let out = replace(
        r"(\w+)@(\w+)",
        RegexOptions::default(),
        "bob@ex",
        "$2 at ${1}",
    )
    .unwrap();
    assert_eq!(out, "ex at bob");
}
