use dupot_dev_tools::domain::feature::formating::xml_format::format;

#[test]
fn indents_and_adds_declaration() {
    assert_eq!(
        format("<a><b>x</b><c/></a>").unwrap(),
        "<?xml version=\"1.0\"?>\n<a>\n    <b>x</b>\n    <c/>\n</a>"
    );
}

#[test]
fn keeps_existing_declaration() {
    let out = format("<?xml version=\"1.0\" encoding=\"UTF-8\"?><a/>").unwrap();
    assert!(out.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert_eq!(out.matches("<?xml").count(), 1);
}

#[test]
fn rejects_malformed_or_empty_documents() {
    assert!(format("<a><b></a>").is_err());
    assert!(format("<a>").is_err());
    assert!(format("").is_err());
}
