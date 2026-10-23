//! Published releases of the application, to tell the user when a newer one
//! is out.

/// A release published on the project's page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Release tag, e.g. `2.1.3` or `v2.1.3`.
    pub version: String,
    /// Page to download it from.
    pub url: String,
}

impl Release {
    /// True when this release is strictly newer than `current`. Versions
    /// compare by their numeric `major.minor.patch` parts; a tag that isn't
    /// one (e.g. `v1.0-beta.9`) is never newer.
    pub fn is_newer_than(&self, current: &str) -> bool {
        match (version_parts(&self.version), version_parts(current)) {
            (Some(latest), Some(current)) => latest > current,
            _ => false,
        }
    }
}

fn version_parts(version: &str) -> Option<Vec<u64>> {
    let version = version.trim();
    let version = version.strip_prefix(['v', 'V']).unwrap_or(version);
    version.split('.').map(|part| part.parse().ok()).collect()
}
