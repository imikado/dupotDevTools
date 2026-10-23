use dupot_dev_tools::domain::feature::parsing::xpath_tester::{SAMPLE, evaluate};

#[test]
fn returns_matching_nodes_as_xml() {
    let out = evaluate("//book[price < 9]/title", SAMPLE).unwrap();
    assert_eq!(out, "<!-- 1 match -->\n<title>Le Petit Prince</title>");
    let out = evaluate("//book/@lang", SAMPLE).unwrap();
    assert_eq!(out, "<!-- 2 matches -->\nlang=\"en\"\nlang=\"fr\"");
    let out = evaluate("//book[@id='1']/author", SAMPLE).unwrap();
    assert!(out.ends_with("<author>Frank Herbert</author>"), "{out}");
}

#[test]
fn returns_scalar_values() {
    assert_eq!(evaluate("count(//book)", SAMPLE).unwrap(), "2");
    assert_eq!(evaluate("sum(//price)", SAMPLE).unwrap(), "17.49");
    assert_eq!(
        evaluate("string(//book[2]/title)", SAMPLE).unwrap(),
        "Le Petit Prince"
    );
    assert_eq!(evaluate("boolean(//magazine)", SAMPLE).unwrap(), "false");
}

#[test]
fn reports_errors() {
    assert!(
        evaluate("//book[", SAMPLE)
            .unwrap_err()
            .message()
            .starts_with("Invalid XPath")
    );
    assert!(
        evaluate("//a", "<a>")
            .unwrap_err()
            .message()
            .starts_with("Invalid XML")
    );
    assert!(evaluate(" ", SAMPLE).is_err());
}
