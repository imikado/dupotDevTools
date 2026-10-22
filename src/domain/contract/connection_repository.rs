use std::path::{Path, PathBuf};

use crate::domain::entity::database::ConnectionConfig;
use crate::domain::error::DomainResult;

/// Files holding SQL viewer connections; implemented in `infrastructure::api`.
pub trait ConnectionRepository {
    /// Connections saved in `path`, without passwords.
    fn load(&self, path: &Path) -> DomainResult<Vec<ConnectionConfig>>;
    /// Writes `connections` to `path`. Passwords are never written.
    fn save(&self, path: &Path, connections: &[ConnectionConfig]) -> DomainResult<()>;
    /// The file last opened or saved, reopened at startup.
    fn last_path(&self) -> Option<PathBuf>;
    fn set_last_path(&self, path: &Path) -> DomainResult<()>;
}
