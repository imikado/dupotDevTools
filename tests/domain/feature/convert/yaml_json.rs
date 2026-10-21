use dupot_dev_tools::domain::feature::convert::yaml_json::{json_to_yaml, yaml_to_json};

#[test]
fn yaml_to_json_keeps_key_order_and_unicode() {
    let json = yaml_to_json("b: 1\na: [x, é]\n").unwrap();
    assert_eq!(
        json,
        "{\n    \"b\": 1,\n    \"a\": [\n        \"x\",\n        \"é\"\n    ]\n}"
    );
}

#[test]
fn json_to_yaml_round_trip() {
    let json = yaml_to_json("b: 1\na: [x, é]\n").unwrap();
    assert_eq!(json_to_yaml(&json).unwrap(), "b: 1\na:\n- x\n- é\n");
}

#[test]
fn invalid_inputs_are_errors() {
    assert!(yaml_to_json("a: [").is_err());
    assert!(json_to_yaml("{").is_err());
}
