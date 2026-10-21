use dupot_dev_tools::domain::feature::encoding::jwt_decode::decode;

const TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
    eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.\
    SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

#[test]
fn decodes_header_and_payload() {
    let out = decode(TOKEN).unwrap();
    assert!(out.starts_with("{\n    \"header\": {\n        \"alg\": \"HS256\""));
    assert!(out.contains("\"name\": \"John Doe\""));
}

#[test]
fn surrounding_whitespace_is_ignored() {
    assert!(decode(&format!("  {TOKEN}\n")).is_ok());
}

#[test]
fn token_without_dot_is_rejected() {
    let err = decode("nope").unwrap_err();
    assert_eq!(
        err.message(),
        "not a valid JWT (expected header.payload.signature)"
    );
}

#[test]
fn non_json_part_is_an_error() {
    assert!(decode("aGVsbG8.aGVsbG8.sig").is_err());
}
