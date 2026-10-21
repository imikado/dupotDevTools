use dupot_dev_tools::domain::feature::encoding::url_encoding::{decode, encode};

#[test]
fn keeps_unreserved_characters_only() {
    assert_eq!(encode("a-b_c.d~e f/g?é"), "a-b_c.d~e%20f%2Fg%3F%C3%A9");
}

#[test]
fn decodes_percent_sequences() {
    assert_eq!(decode("e%20f%2Fg%3F%C3%A9"), "e f/g?é");
}

#[test]
fn invalid_utf8_is_replaced() {
    assert_eq!(decode("%FF"), "\u{FFFD}");
}
