use dupot_dev_tools::domain::entity::release::Release;

fn release(version: &str) -> Release {
    Release {
        version: version.into(),
        url: String::new(),
    }
}

#[test]
fn newer_patch_minor_and_major() {
    assert!(release("2.1.3").is_newer_than("2.1.2"));
    assert!(release("2.2.0").is_newer_than("2.1.9"));
    assert!(release("3.0.0").is_newer_than("2.10.0"));
}

#[test]
fn same_or_older_is_not_newer() {
    assert!(!release("2.1.2").is_newer_than("2.1.2"));
    assert!(!release("2.1.1").is_newer_than("2.1.2"));
}

#[test]
fn numbers_compare_numerically() {
    assert!(release("2.10.0").is_newer_than("2.9.0"));
}

#[test]
fn v_prefix_is_ignored() {
    assert!(release("v2.1.3").is_newer_than("2.1.2"));
}

#[test]
fn unparsable_tag_is_never_newer() {
    assert!(!release("v1.0-beta.9").is_newer_than("0.1.0"));
    assert!(!release("2.1.3").is_newer_than("dev"));
}
