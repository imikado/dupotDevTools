use dupot_dev_tools::domain::entity::app_info::{APP, BuildInfo};

const BUILD: BuildInfo = BuildInfo {
    version: "1.2.3",
    date: Some("2026-10-21"),
    package: "appimage",
    os: "linux",
    arch: "x86_64",
};

#[test]
fn version_label_includes_date() {
    assert_eq!(BUILD.version_label(), "1.2.3 (2026-10-21)");
}

#[test]
fn version_label_without_metadata() {
    let bare = BuildInfo {
        date: None,
        ..BUILD
    };
    assert_eq!(bare.version_label(), "1.2.3");
}

#[test]
fn details_for_bug_reports() {
    assert_eq!(
        BUILD.details(),
        "Version: 1.2.3\nBuild date: 2026-10-21\nPackage: appimage\nPlatform: linux-x86_64"
    );
    let unknown = BuildInfo {
        date: None,
        ..BUILD
    };
    assert!(unknown.details().contains("Build date: unknown"));
}

#[test]
fn current_build_matches_cargo_toml_and_dupot_site() {
    assert_eq!(APP.build.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(APP.website, "https://www.dupot.org");
    let date = APP.build.date.expect("build.rs always sets a date");
    assert_eq!(date.len(), 10, "{date}");
    assert_eq!(&date[4..5], "-");
}
