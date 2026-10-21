//! Identity and version of the application, shown in the About dialog so a
//! user can tell exactly which build they run.
//!
//! The version number comes from `Cargo.toml`; build date and package type
//! are captured at compile time by `build.rs`.

/// Where and how this binary was built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildInfo {
    pub version: &'static str,
    /// Build date, `YYYY-MM-DD`.
    pub date: Option<&'static str>,
    /// `flatpak`, `appimage`, `windows`, `nix` or `source`.
    pub package: &'static str,
    pub os: &'static str,
    pub arch: &'static str,
}

impl BuildInfo {
    /// `1.0.0 (2026-10-21)`, or just `1.0.0` without a build date.
    pub fn version_label(&self) -> String {
        match self.date {
            Some(date) => format!("{} ({date})", self.version),
            None => self.version.to_string(),
        }
    }

    /// Multi-line details for bug reports.
    pub fn details(&self) -> String {
        let platform = format!("{}-{}", self.os, self.arch);
        [
            ("Version", self.version),
            ("Build date", self.date.unwrap_or("unknown")),
            ("Package", self.package),
            ("Platform", platform.as_str()),
        ]
        .iter()
        .map(|(label, value)| format!("{label}: {value}"))
        .collect::<Vec<_>>()
        .join("\n")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub developer: &'static str,
    pub website: &'static str,
    pub issue_url: &'static str,
    pub build: BuildInfo,
}

/// This application, as compiled.
pub const APP: AppInfo = AppInfo {
    id: "org.dupot.devtools",
    name: "Dupot Dev Tools",
    developer: "Michael Bertocchi",
    website: "https://www.dupot.org",
    issue_url: "https://github.com/imikado/dupotDevTools/issues",
    build: BuildInfo {
        version: env!("CARGO_PKG_VERSION"),
        date: option_env!("DUPOT_BUILD_DATE"),
        package: env!("DUPOT_BUILD_PACKAGE"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
    },
};
