use crate::domain::entity::release::Release;
use crate::domain::error::DomainResult;

/// Where releases are published; implemented in `infrastructure::api`.
pub trait ReleaseRepository {
    /// The latest stable release. Blocks on the network.
    fn latest_release(&self) -> DomainResult<Release>;
}
