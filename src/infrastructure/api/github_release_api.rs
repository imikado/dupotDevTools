//! GitHub implementation of the domain's `ReleaseRepository`.

use std::time::Duration;

use serde_json::Value;

use crate::domain::contract::release_repository::ReleaseRepository;
use crate::domain::entity::app_info::APP;
use crate::domain::entity::release::Release;
use crate::domain::error::{DomainError, DomainResult};

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/imikado/dupotDevTools/releases/latest";
const TIMEOUT: Duration = Duration::from_secs(10);

pub struct GithubReleaseApi;

impl ReleaseRepository for GithubReleaseApi {
    fn latest_release(&self) -> DomainResult<Release> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .build()
            .into();
        let body = agent
            .get(LATEST_RELEASE_URL)
            // GitHub rejects requests without a User-Agent.
            .header("User-Agent", format!("{}/{}", APP.id, APP.build.version))
            .header("Accept", "application/vnd.github+json")
            .call()?
            .body_mut()
            .read_to_string()?;
        let json: Value = serde_json::from_str(&body)?;
        let field = |name: &str| {
            json.get(name)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| DomainError::new(format!("Release without `{name}`.")))
        };
        Ok(Release {
            version: field("tag_name")?,
            url: field("html_url")?,
        })
    }
}
