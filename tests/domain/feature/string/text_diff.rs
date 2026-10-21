use dupot_dev_tools::domain::feature::string::text_diff::{DiffOptions, diff};

const CONTEXT: DiffOptions = DiffOptions {
    ignore_whitespace: false,
    ignore_case: false,
    context_lines: 3,
};

#[test]
fn identical_texts() {
    let (unified, stats) = diff("a\nb\n", "a\nb\n", CONTEXT);
    assert!(unified.is_empty());
    assert!(stats.is_identical());
    assert_eq!(stats.summary(), "No differences");
}

#[test]
fn unified_diff_and_stats() {
    let (unified, stats) = diff("a\nb\nc", "a\nB\nc\nd", CONTEXT);
    assert_eq!(
        unified,
        "--- original\n+++ modified\n@@ -1,3 +1,4 @@\n a\n-b\n+B\n c\n+d\n"
    );
    assert_eq!((stats.added, stats.removed, stats.unchanged), (2, 1, 2));
    assert_eq!(stats.summary(), "+2 added, −1 removed, 2 unchanged");
}

#[test]
fn ignore_options() {
    let opts = DiffOptions {
        ignore_whitespace: true,
        ignore_case: true,
        ..CONTEXT
    };
    let (_, stats) = diff("  Hello \nWorld", "hello\n\tworld", opts);
    assert!(stats.is_identical());
}

#[test]
fn context_lines_limit_the_output() {
    let old = "1\n2\n3\n4\n5\n6\n7";
    let new = "1\n2\n3\nX\n5\n6\n7";
    let (narrow, _) = diff(
        old,
        new,
        DiffOptions {
            context_lines: 1,
            ..CONTEXT
        },
    );
    assert!(narrow.contains(" 3\n-4\n+X\n 5"));
    assert!(!narrow.contains(" 2\n"));
}
