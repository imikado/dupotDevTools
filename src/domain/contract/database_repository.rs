use crate::domain::entity::database::{ConnectionConfig, DbKind, QueryResult};
use crate::domain::error::DomainResult;

/// Access to a SQL database; implemented in `infrastructure::api`.
pub trait DatabaseRepository {
    /// Closes any open connection, then opens the configured one.
    fn connect(&mut self, config: &ConnectionConfig) -> DomainResult<()>;
    fn is_connected(&self) -> bool;
    fn kind(&self) -> Option<DbKind>;
    /// Runs a query and returns its columns and rows, every value as text.
    fn execute(&mut self, sql: &str) -> DomainResult<QueryResult>;
    fn list_tables(&mut self) -> DomainResult<Vec<String>>;
    /// `(column name, column type)` of a table reported by `list_tables`.
    fn list_columns(&mut self, table: &str) -> DomainResult<Vec<(String, String)>>;
    fn close(&mut self);
}
