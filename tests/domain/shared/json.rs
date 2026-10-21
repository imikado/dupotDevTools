use dupot_dev_tools::domain::shared::json::to_pretty_json;
use serde_json::json;

#[test]
fn indents_with_four_spaces_and_keeps_unicode() {
    let out = to_pretty_json(&json!({"a": ["é"]})).unwrap();
    assert_eq!(out, "{\n    \"a\": [\n        \"é\"\n    ]\n}");
}
