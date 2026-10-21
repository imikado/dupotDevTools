mod database;

use std::collections::HashSet;

use dupot_dev_tools::infrastructure::ui::features::registry;

#[test]
fn names_are_unique() {
    let features = registry();
    let names: HashSet<_> = features.iter().map(|f| f.entity.name).collect();
    assert_eq!(names.len(), features.len());
}

#[test]
fn every_feature_has_metadata_and_an_icon() {
    for f in registry() {
        assert!(!f.entity.title.is_empty(), "{}", f.entity.name);
        assert!(!f.entity.description.is_empty(), "{}", f.entity.name);
        let icon = f.icon.unwrap_or_default();
        assert!(
            icon.starts_with(b"\x89PNG"),
            "{} icon is not a PNG",
            f.entity.name
        );
    }
}

#[test]
fn entity_matches_its_folder() {
    // The registry macro takes the icon from `<section>/<name>/icon.png`, so a
    // mismatching ENTITY would pair a tool with another tool's folder.
    let src = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/infrastructure/ui/features"
    );
    for f in registry() {
        let dir = format!("{src}/{}/{}", f.entity.section, f.entity.name);
        assert!(std::path::Path::new(&dir).is_dir(), "missing {dir}");
    }
}
