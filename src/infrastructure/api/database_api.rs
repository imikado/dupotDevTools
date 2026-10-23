//! SQLite / PostgreSQL / MySQL implementation of the domain's
//! `DatabaseRepository`.

use std::sync::Arc;

use mysql::prelude::Queryable;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};

use crate::domain::contract::database_repository::DatabaseRepository;
use crate::domain::entity::database::{Cell, ConnectionConfig, DbKind, ForeignKey, QueryResult};
use crate::domain::error::{DomainError, DomainResult};

enum Connection {
    Sqlite(rusqlite::Connection),
    Postgresql(postgres::Client),
    Mysql(mysql::Conn),
}

#[derive(Default)]
pub struct DatabaseApi {
    conn: Option<Connection>,
}

impl DatabaseRepository for DatabaseApi {
    fn connect(&mut self, config: &ConnectionConfig) -> DomainResult<()> {
        self.close();
        let conn = match config.kind {
            DbKind::Sqlite => {
                let path = config.param("path");
                if path.is_empty() {
                    return Err(DomainError::new("A database file path is required."));
                }
                Connection::Sqlite(rusqlite::Connection::open(path)?)
            }
            DbKind::Postgresql => {
                let mut pg = postgres::Config::new();
                pg.host(config.host())
                    .port(config.port()?)
                    .dbname(config.param("database"))
                    .user(config.param("user"))
                    .password(config.param("password"));
                Connection::Postgresql(pg.connect(postgres_tls()?)?)
            }
            DbKind::Mysql => {
                let database = config.param("database");
                let opts = mysql::OptsBuilder::new()
                    .ip_or_hostname(Some(config.host()))
                    .tcp_port(config.port()?)
                    .db_name((!database.is_empty()).then_some(database))
                    .user(Some(config.param("user")))
                    .pass(Some(config.param("password")));
                Connection::Mysql(mysql::Conn::new(opts)?)
            }
        };
        self.conn = Some(conn);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    fn kind(&self) -> Option<DbKind> {
        self.conn.as_ref().map(|c| match c {
            Connection::Sqlite(_) => DbKind::Sqlite,
            Connection::Postgresql(_) => DbKind::Postgresql,
            Connection::Mysql(_) => DbKind::Mysql,
        })
    }

    fn execute(&mut self, sql: &str) -> DomainResult<QueryResult> {
        match self
            .conn
            .as_mut()
            .ok_or_else(|| DomainError::new("Not connected."))?
        {
            Connection::Sqlite(c) => Ok(sqlite_execute(c, sql)?),
            Connection::Postgresql(c) => Ok(postgres_execute(c, sql)?),
            Connection::Mysql(c) => Ok(mysql_execute(c, sql)?),
        }
    }

    fn list_tables(&mut self) -> DomainResult<Vec<String>> {
        let sql = match self.kind() {
            None => return Ok(vec![]),
            Some(DbKind::Sqlite) => {
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            }
            Some(DbKind::Postgresql) => {
                "SELECT table_name FROM information_schema.tables \
                 WHERE table_schema='public' ORDER BY table_name"
            }
            Some(DbKind::Mysql) => "SHOW TABLES",
        };
        let (_, rows) = self.execute(sql)?;
        Ok(rows
            .into_iter()
            .filter_map(|r| r.into_iter().next().flatten())
            .collect())
    }

    fn list_columns(&mut self, table: &str) -> DomainResult<Vec<(String, String)>> {
        let sql = match self.kind() {
            None => return Ok(vec![]),
            Some(DbKind::Sqlite) => format!("PRAGMA table_info('{}')", table.replace('\'', "''")),
            Some(DbKind::Postgresql) => format!(
                "SELECT column_name::text, data_type::text FROM information_schema.columns \
                 WHERE table_name = '{}' ORDER BY ordinal_position",
                table.replace('\'', "''")
            ),
            Some(DbKind::Mysql) => format!("SHOW COLUMNS FROM `{}`", table.replace('`', "``")),
        };
        let (name_idx, type_idx) = match self.kind() {
            Some(DbKind::Sqlite) => (1, 2),
            _ => (0, 1),
        };
        let (_, rows) = self.execute(&sql)?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let name = r.get(name_idx).cloned().flatten()?;
                let kind = r.get(type_idx).cloned().flatten().unwrap_or_default();
                Some((name, kind))
            })
            .collect())
    }

    fn list_foreign_keys(&mut self, table: &str) -> DomainResult<Vec<ForeignKey>> {
        let quoted = table.replace('\'', "''");
        let (sql, idx) = match self.kind() {
            None => return Ok(vec![]),
            // id, seq, table, from, to, …
            Some(DbKind::Sqlite) => (format!("PRAGMA foreign_key_list('{quoted}')"), (3, 2, 4)),
            Some(DbKind::Postgresql) => (
                format!(
                    "SELECT kcu.column_name::text, ccu.table_name::text, ccu.column_name::text \
                     FROM information_schema.table_constraints tc \
                     JOIN information_schema.key_column_usage kcu \
                       ON kcu.constraint_name = tc.constraint_name \
                      AND kcu.table_schema = tc.table_schema \
                     JOIN information_schema.constraint_column_usage ccu \
                       ON ccu.constraint_name = tc.constraint_name \
                      AND ccu.table_schema = tc.table_schema \
                     WHERE tc.constraint_type = 'FOREIGN KEY' \
                       AND tc.table_schema = 'public' AND tc.table_name = '{quoted}'"
                ),
                (0, 1, 2),
            ),
            Some(DbKind::Mysql) => (
                format!(
                    "SELECT COLUMN_NAME, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
                     FROM information_schema.KEY_COLUMN_USAGE \
                     WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '{quoted}' \
                       AND REFERENCED_TABLE_NAME IS NOT NULL"
                ),
                (0, 1, 2),
            ),
        };
        let (column, ref_table, ref_column) = idx;
        let (_, rows) = self.execute(&sql)?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                Some(ForeignKey {
                    column: r.get(column).cloned().flatten()?,
                    ref_table: r.get(ref_table).cloned().flatten()?,
                    // SQLite leaves `to` NULL when the key targets the primary key.
                    ref_column: r.get(ref_column).cloned().flatten()?,
                })
            })
            .collect())
    }

    fn close(&mut self) {
        self.conn = None;
    }
}

fn sqlite_execute(conn: &rusqlite::Connection, sql: &str) -> rusqlite::Result<QueryResult> {
    use rusqlite::types::ValueRef;
    let mut stmt = conn.prepare(sql)?;
    let columns: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
    let n = columns.len();
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let mut cells = Vec::with_capacity(n);
        for i in 0..n {
            cells.push(match row.get_ref(i)? {
                ValueRef::Null => None,
                ValueRef::Integer(v) => Some(v.to_string()),
                ValueRef::Real(v) => Some(format!("{v:?}")),
                ValueRef::Text(v) => Some(String::from_utf8_lossy(v).into_owned()),
                ValueRef::Blob(v) => Some(format!("<{} bytes>", v.len())),
            });
        }
        out.push(cells);
    }
    Ok((columns, out))
}

/// TLS for PostgreSQL. The default `sslmode=prefer` tries TLS first and falls
/// back to plain text, so servers that require encryption (`hostssl` lines in
/// pg_hba.conf) accept the connection. Like libpq's `prefer` / `require`, the
/// server certificate is not verified: dev servers mostly use self-signed ones.
fn postgres_tls() -> DomainResult<tokio_postgres_rustls::MakeRustlsConnect> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(|e| DomainError::new(format!("TLS setup failed: {e}")))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyCert(provider)))
        .with_no_client_auth();
    Ok(tokio_postgres_rustls::MakeRustlsConnect::new(config))
}

/// Accepts any server certificate but still checks the handshake signatures.
#[derive(Debug)]
struct AcceptAnyCert(Arc<CryptoProvider>);

impl ServerCertVerifier for AcceptAnyCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn postgres_execute(
    client: &mut postgres::Client,
    sql: &str,
) -> Result<QueryResult, postgres::Error> {
    use postgres::SimpleQueryMessage;
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    for msg in client.simple_query(sql)? {
        match msg {
            SimpleQueryMessage::RowDescription(cols) => {
                columns = cols.iter().map(|c| c.name().to_string()).collect();
                rows.clear();
            }
            SimpleQueryMessage::Row(row) => {
                rows.push(
                    (0..row.len())
                        .map(|i| row.get(i).map(String::from))
                        .collect(),
                );
            }
            _ => {}
        }
    }
    Ok((columns, rows))
}

fn mysql_value_to_string(value: mysql::Value) -> Cell {
    use mysql::Value;
    Some(match value {
        Value::NULL => return None,
        Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
        Value::Int(v) => v.to_string(),
        Value::UInt(v) => v.to_string(),
        Value::Float(v) => format!("{v:?}"),
        Value::Double(v) => format!("{v:?}"),
        Value::Date(y, m, d, h, mi, s, us) => {
            format!("{y:04}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02}.{us:06}")
        }
        Value::Time(neg, d, h, mi, s, us) => format!(
            "{}{}:{mi:02}:{s:02}.{us:06}",
            if neg { "-" } else { "" },
            d * 24 + u32::from(h)
        ),
    })
}

fn mysql_execute(conn: &mut mysql::Conn, sql: &str) -> mysql::Result<QueryResult> {
    let mut result = conn.query_iter(sql)?;
    let mut columns = Vec::new();
    let mut rows = Vec::new();
    if let Some(set) = result.iter() {
        columns = set
            .columns()
            .as_ref()
            .iter()
            .map(|c| c.name_str().into_owned())
            .collect();
        for row in set {
            rows.push(
                row?.unwrap()
                    .into_iter()
                    .map(mysql_value_to_string)
                    .collect(),
            );
        }
    }
    Ok((columns, rows))
}
