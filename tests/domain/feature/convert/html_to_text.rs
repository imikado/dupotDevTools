use dupot_dev_tools::domain::feature::convert::html_to_text::convert;

#[test]
fn strips_tags() {
    let out = convert("<p>Hello <b>world</b></p>").unwrap();
    assert!(out.contains("Hello"));
    assert!(out.contains("world"));
    assert!(!out.contains('<'));
}

#[test]
fn empty_input_gives_empty_text() {
    assert_eq!(convert("").unwrap().trim(), "");
}
