use dupot_dev_tools::domain::shared::format::{number, rows};

#[test]
fn rows_align_values_after_the_longest_label() {
    assert_eq!(
        rows(&[("a", "1"), ("long", "2"), ("x", "")]),
        "a     1\nlong  2\nx"
    );
}

#[test]
fn numbers_drop_trailing_zeros() {
    assert_eq!(number(1.5, 3), "1.5");
    assert_eq!(number(2.0, 3), "2");
    assert_eq!(number(-0.0001, 2), "-1.000e-4");
    assert_eq!(number(-0.0, 2), "0");
}
