//! SQLite / PostgreSQL / MySQL implementation of the domain's
//! `DatabaseRepository`.

use mysql::prelude::Queryable;

use crate::domain::contract::database_repository::DatabaseRepository;
use crate::domain::entity::database::{Cell, ConnectionConfig, DbKind, QueryResult};
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
                Connection::Postgresql(pg.connect(postgres::NoTls)?)
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
