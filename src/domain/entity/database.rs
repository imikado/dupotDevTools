use std::collections::BTreeMap;

use crate::domain::error::{DomainError, DomainResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbKind {
    Sqlite,
    Postgresql,
    Mysql,
}

impl DbKind {
    pub const ALL: [DbKind; 3] = [DbKind::Sqlite, DbKind::Postgresql, DbKind::Mysql];

    pub fn label(self) -> &'static str {
        match self {
            DbKind::Sqlite => "SQLite",
            DbKind::Postgresql => "PostgreSQL",
            DbKind::Mysql => "MySQL",
        }
    }

    /// Stable identifier used when saving connections.
    pub fn id(self) -> &'static str {
        match self {
            DbKind::Sqlite => "sqlite",
            DbKind::Postgresql => "postgresql",
            DbKind::Mysql => "mysql",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.id() == id)
    }

    pub fn default_port(self) -> &'static str {
        match self {
            DbKind::Sqlite => "",
            DbKind::Postgresql => "5432",
            DbKind::Mysql => "3306",
        }
    }
}

pub type ConnectionParams = BTreeMap<&'static str, String>;

/// Every key a `ConnectionParams` may hold.
pub const PARAM_KEYS: [&str; 6] = ["path", "host", "port", "database", "user", PASSWORD];

const PASSWORD: &str = "password";

/// One result cell rendered as text; `None` is SQL NULL.
pub type Cell = Option<String>;

/// (column names, rows)
pub type QueryResult = (Vec<String>, Vec<Vec<Cell>>);

/// `column` of a table references `ref_table.ref_column`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignKey {
    pub column: String,
    pub ref_table: String,
    pub ref_column: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub name: String,
    pub kind: DbKind,
    pub params: ConnectionParams,
}

/// Raw values typed in the "New connection" form.
#[derive(Clone, Debug, Default)]
pub struct ConnectionForm {
    pub name: String,
    pub path: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl ConnectionForm {
    /// Validates the form and fills in defaults (name `connection-N`, host
    /// `localhost`, the kind's default port). `existing` is the number of
    /// connections already defined.
    pub fn into_config(self, kind: DbKind, existing: usize) -> DomainResult<ConnectionConfig> {
        let name = match self.name.trim() {
            "" => format!("connection-{}", existing + 1),
            n => n.to_string(),
        };
        let or_default = |v: &str, d: &str| match v.trim() {
            "" => d.to_string(),
            v => v.to_string(),
        };

        let mut params = ConnectionParams::new();
        if kind == DbKind::Sqlite {
            let path = self.path.trim();
            if path.is_empty() {
                return Err(DomainError::new("A database file path is required."));
            }
            params.insert("path", path.to_string());
        } else {
            params.insert("host", or_default(&self.host, "localhost"));
            params.insert("port", or_default(&self.port, kind.default_port()));
            params.insert("database", self.database.trim().to_string());
            params.insert("user", self.user.trim().to_string());
            params.insert("password", self.password);
        }
        Ok(ConnectionConfig { name, kind, params })
    }
}

impl ConnectionConfig {
    pub fn param(&self, key: &str) -> &str {
        self.params.get(key).map(String::as_str).unwrap_or("")
    }

    /// True for a server connection whose password is unknown: passwords are
    /// never saved, so they are asked again after a restart.
    pub fn needs_password(&self) -> bool {
        self.kind != DbKind::Sqlite && !self.params.contains_key(PASSWORD)
    }

    pub fn set_password(&mut self, password: String) {
        self.params.insert(PASSWORD, password);
    }

    pub fn forget_password(&mut self) {
        self.params.remove(PASSWORD);
    }

    /// The params safe to write to disk: all of them but the password.
    pub fn persisted_params(&self) -> impl Iterator<Item = (&'static str, &str)> {
        self.params
            .iter()
            .filter(|(k, _)| **k != PASSWORD)
            .map(|(k, v)| (*k, v.as_str()))
    }

    pub fn host(&self) -> &str {
        match self.param("host") {
            "" => "localhost",
            h => h,
        }
    }

    pub fn port(&self) -> DomainResult<u16> {
        let raw = match self.param("port") {
            "" => self.kind.default_port(),
            p => p,
        };
        raw.parse()
            .map_err(|_| DomainError::new(format!("Invalid port: {raw}")))
    }
}
