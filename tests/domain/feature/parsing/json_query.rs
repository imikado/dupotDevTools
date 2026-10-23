use dupot_dev_tools::domain::feature::parsing::json_query::{QueryLanguage, SAMPLE, run};

#[test]
fn runs_json_path() {
    let out = run(
        QueryLanguage::JsonPath,
        "$.store.book[?@.price < 10].title",
        SAMPLE,
    )
    .unwrap();
    assert_eq!(
        out,
        "[\n    \"Sayings of the Century\",\n    \"Moby Dick\"\n]"
    );
    assert_eq!(
        run(QueryLanguage::JsonPath, "$.nothing", SAMPLE).unwrap(),
        "[]"
    );
    assert!(run(QueryLanguage::JsonPath, "$.[", SAMPLE).is_err());
    assert!(run(QueryLanguage::JsonPath, "$", "{oops").is_err());
}

#[test]
fn runs_jq() {
    let out = run(
        QueryLanguage::Jq,
        ".store.book[] | select(.price < 10) | .title",
        SAMPLE,
    )
    .unwrap();
    assert_eq!(out, "\"Sayings of the Century\"\n\"Moby Dick\"");
    let out = run(QueryLanguage::Jq, "{n: (.store.book | length)}", SAMPLE).unwrap();
    assert_eq!(out, "{\n    \"n\": 4\n}");
    // Each JSON value of the input is filtered.
    assert_eq!(run(QueryLanguage::Jq, ". * 2", "1 2").unwrap(), "2\n4");
}

#[test]
fn reports_jq_errors() {
    assert!(
        run(QueryLanguage::Jq, ".a |", "{}")
            .unwrap_err()
            .message()
            .contains("expected")
    );
    assert!(
        run(QueryLanguage::Jq, "nope", "{}")
            .unwrap_err()
            .message()
            .contains("nope")
    );
    assert!(
        run(QueryLanguage::Jq, "error(\"boom\")", "{}")
            .unwrap_err()
            .message()
            .contains("boom")
    );
    assert!(run(QueryLanguage::Jq, ".", "{oops").is_err());
    assert!(run(QueryLanguage::Jq, "  ", "{}").is_err());
}

#[test]
fn caps_endless_jq_output() {
    let out = run(QueryLanguage::Jq, "repeat(1)", "null").unwrap();
    assert_eq!(out.lines().count(), 10_000);
}
