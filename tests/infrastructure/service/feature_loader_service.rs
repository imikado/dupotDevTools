use dupot_dev_tools::infrastructure::service::feature_loader_service::FeatureLoaderService;

#[test]
fn sections_are_sorted_and_unique() {
    let loader = FeatureLoaderService::default();
    let sections = loader.sections();
    let mut sorted = sections.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sections, sorted);
    assert!(sections.contains(&"encoding"));
}

#[test]
fn features_in_section_are_filtered_and_sorted() {
    let loader = FeatureLoaderService::default();
    let names: Vec<_> = loader
        .features_in_section("encoding")
        .iter()
        .map(|f| f.entity.name)
        .collect();
    assert_eq!(
        names,
        [
            "base64_encoding",
            "html_encoding",
            "jwt_decode",
            "jwt_sign",
            "url_encoding"
        ]
    );
}

#[test]
fn all_features_cover_every_section() {
    let loader = FeatureLoaderService::default();
    let total: usize = loader
        .sections()
        .iter()
        .map(|s| loader.features_in_section(s).len())
        .sum();
    assert_eq!(total, loader.all_features().len());
}
