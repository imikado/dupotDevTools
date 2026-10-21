use dupot_dev_tools::domain::feature::formating::json_format::{minify, prettify};

#[test]
fn prettify_indents_and_keeps_key_order() {
    assert_eq!(
        prettify(r#"{"b":1,"a":[true,null]}"#).unwrap(),
        "{\n    \"b\": 1,\n    \"a\": [\n        true,\n        null\n    ]\n}"
    );
}

#[test]
fn minify_removes_whitespace() {
    assert_eq!(
        minify("{ \"b\" : 1,\n \"a\" : [ 2 ] }").unwrap(),
        r#"{"b":1,"a":[2]}"#
    );
}

#[test]
fn invalid_json_is_an_error() {
    assert!(prettify("{").is_err());
    assert!(minify("[1,").is_err());
}
