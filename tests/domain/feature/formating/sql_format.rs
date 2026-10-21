use dupot_dev_tools::domain::feature::formating::sql_format::format;

#[test]
fn uppercases_keywords_and_reindents() {
    let out = format("select id, name from users where id = 1");
    assert!(out.starts_with("SELECT"));
    assert!(out.contains("\nFROM\n    users"));
    assert!(out.contains("\nWHERE\n    id = 1"));
}
