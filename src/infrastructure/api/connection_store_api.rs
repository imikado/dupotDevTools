//! JSON file implementation of the domain's `ConnectionRepository`.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::domain::contract::connection_repository::ConnectionRepository;
use crate::domain::entity::app_info::APP;
use crate::domain::entity::database::{ConnectionConfig, ConnectionParams, DbKind, PARAM_KEYS};
use crate::domain::error::DomainResult;
use crate::domain::shared::json::to_pretty_json;

/// Remembers the last file in `state_dir`.
pub struct ConnectionStoreApi {
    state_dir: PathBuf,
}

impl ConnectionStoreApi {
    pub fn new(state_dir: impl Into<PathBuf>) -> Self {
        Self {
            state_dir: state_dir.into(),
        }
    }

    fn last_path_file(&self) -> PathBuf {
        self.state_dir.join("sql_viewer_last_file")
    }
}

impl Default for ConnectionStoreApi {
    /// `$XDG_CONFIG_HOME/<app id>/`.
    fn default() -> Self {
        Self::new(gtk::glib::user_config_dir().join(APP.id))
    }
}

impl ConnectionRepository for ConnectionStoreApi {
    fn load(&self, path: &Path) -> DomainResult<Vec<ConnectionConfig>> {
        let entries: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        Ok(entries.iter().filter_map(config_from_json).collect())
    }

    fn save(&self, path: &Path, connections: &[ConnectionConfig]) -> DomainResult<()> {
        let entries: Vec<Value> = connections
            .iter()
            .map(|c| {
                let params: Map<String, Value> = c
                    .persisted_params()
                    .map(|(k, v)| (k.to_string(), v.into()))
                    .collect();
                json!({ "name": c.name, "kind": c.kind.id(), "params": params })
            })
            .collect();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, to_pretty_json(&entries)?)?;
        Ok(())
    }

    fn last_path(&self) -> Option<PathBuf> {
        let remembered = std::fs::read_to_string(self.last_path_file())
            .ok()
            .map(|text| PathBuf::from(text.trim()));
        // Before files could be chosen, connections were saved here.
        let legacy = self.state_dir.join("sql_connections.json");
        remembered
            .into_iter()
            .chain([legacy])
            .find(|path| path.is_file())
    }

    fn set_last_path(&self, path: &Path) -> DomainResult<()> {
        std::fs::create_dir_all(&self.state_dir)?;
        std::fs::write(self.last_path_file(), path.to_string_lossy().as_bytes())?;
        Ok(())
    }
}

/// Skips malformed entries and unknown params rather than failing the load.
fn config_from_json(entry: &Value) -> Option<ConnectionConfig> {
    let name = entry.get("name")?.as_str()?.to_string();
    let kind = DbKind::from_id(entry.get("kind")?.as_str()?)?;
    let mut params = ConnectionParams::new();
    for (key, value) in entry.get("params")?.as_object()? {
        if let (Some(key), Some(value)) =
            (PARAM_KEYS.into_iter().find(|k| k == key), value.as_str())
        {
            params.insert(key, value.to_string());
        }
    }
    let mut config = ConnectionConfig { name, kind, params };
    config.forget_password();
    Some(config)
}
