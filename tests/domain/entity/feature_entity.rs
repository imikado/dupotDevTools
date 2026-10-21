use dupot_dev_tools::domain::entity::feature_entity::FeatureEntity;

const ENTITY: FeatureEntity = FeatureEntity {
    name: "jwt_decode",
    section: "encoding",
    title: "JWT Decoder",
    description: "Decode JWT tokens to JSON",
};

#[test]
fn matches_title_description_name_and_section() {
    assert!(ENTITY.matches("jwt"));
    assert!(ENTITY.matches("tokens"));
    assert!(ENTITY.matches("jwt_decode"));
    assert!(ENTITY.matches("encoding"));
}

#[test]
fn matching_ignores_case_of_the_entity() {
    assert!(ENTITY.matches("decoder"));
}

#[test]
fn does_not_match_unrelated_text() {
    assert!(!ENTITY.matches("yaml"));
}
