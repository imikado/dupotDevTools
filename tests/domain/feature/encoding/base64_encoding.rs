use dupot_dev_tools::domain::feature::encoding::base64_encoding::{decode, encode};

#[test]
fn encodes_utf8_text() {
    assert_eq!(encode("héllo"), "aMOpbGxv");
}

#[test]
fn decodes_and_ignores_surrounding_whitespace() {
    assert_eq!(decode("  aMOpbGxv\n").unwrap(), "héllo");
}

#[test]
fn invalid_base64_is_an_error() {
    assert!(decode("not base64!").is_err());
}

#[test]
fn non_utf8_payload_is_an_error() {
    assert!(decode("/w==").is_err());
}
