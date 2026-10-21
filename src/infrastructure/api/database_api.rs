//! Thin wrapper around SQLite / PostgreSQL / MySQL connections.

use std::collections::BTreeMap;

use mysql::prelude::Queryable;

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

    pub fn default_port(self) -> &'static str {
        match self {
            DbKind::Sqlite => "",
            DbKind::Postgresql => "5432",
            DbKind::Mysql => "3306",
        }
    }
}

pub type ConnectionParams = BTreeMap<&'static str, String>;
pub type Cell = Option<String>;
pub type QueryResult = (Vec<String>, Vec<Vec<Cell>>);

enum Connection {
    Sqlite(rusqlite::Connection),
    Postgresql(postgres::Client),
    Mysql(mysql::Conn),
}

#[derive(Default)]
pub struct DatabaseApi {
    conn: Option<Connection>,
}

fn param<'a>(params: &'a ConnectionParams, key: &str) -> &'a str {
    params.get(key).map(String::as_str).unwrap_or("")
}

fn port(params: &ConnectionParams, kind: DbKind) -> Result<u16, String> {
    let raw = match param(params, "port") {
        "" => kind.default_port(),
        p => p,
    };
    raw.parse().map_err(|_| format!("Invalid port: {raw}"))
}

fn host(params: &ConnectionParams) -> &str {
    match param(params, "host") {
        "" => "localhost",
        h => h,
    }
}

impl DatabaseApi {
    pub fn connect(&mut self, kind: DbKind, params: &ConnectionParams) -> Result<(), String> {
        self.close();
        let conn = match kind {
            DbKind::Sqlite => {
                let path = param(params, "path");
                if path.is_empty() {
                    return Err("A database file path is required.".into());
                }
                Connection::Sqlite(rusqlite::Connection::open(path).map_err(|e| e.to_string())?)
            }
            DbKind::Postgresql => {
                let mut config = postgres::Config::new();
                config
                    .host(host(params))
                    .port(port(params, kind)?)
                    .dbname(param(params, "database"))
                    .user(param(params, "user"))
                    .password(param(params, "password"));
                Connection::Postgresql(config.connect(postgres::NoTls).map_err(|e| e.to_string())?)
            }
            DbKind::Mysql => {
                let database = param(params, "database");
                let opts = mysql::OptsBuilder::new()
                    .ip_or_hostname(Some(host(params)))
                    .tcp_port(port(params, kind)?)
                    .db_name((!database.is_empty()).then_some(database))
                    .user(Some(param(params, "user")))
                    .pass(Some(param(params, "password")));
                Connection::Mysql(mysql::Conn::new(opts).map_err(|e| e.to_string())?)
            }
        };
        self.conn = Some(conn);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    pub fn kind(&self) -> Option<DbKind> {
        self.conn.as_ref().map(|c| match c {
            Connection::Sqlite(_) => DbKind::Sqlite,
            Connection::Postgresql(_) => DbKind::Postgresql,
            Connection::Mysql(_) => DbKind::Mysql,
        })
    }

    /// Runs a query and returns (column_names, rows), every value rendered as text.
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult, String> {
        match self.conn.as_mut().ok_or("Not connected.")? {
            Connection::Sqlite(c) => sqlite_execute(c, sql).map_err(|e| e.to_string()),
            Connection::Postgresql(c) => postgres_execute(c, sql).map_err(|e| e.to_string()),
            Connection::Mysql(c) => mysql_execute(c, sql).map_err(|e| e.to_string()),
        }
    }

    pub fn list_tables(&mut self) -> Result<Vec<String>, String> {
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

    /// Returns [(column_name, column_type), ...] for a table already reported
    /// by list_tables() (trusted, not free-typed user input).
    pub fn list_columns(&mut self, table: &str) -> Result<Vec<(String, String)>, String> {
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

    pub fn close(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_tables_columns_and_rows() {
        let mut db = DatabaseApi::default();
        let params = ConnectionParams::from([("path", ":memory:".to_string())]);
        db.connect(DbKind::Sqlite, &params).unwrap();
        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, score REAL)")
            .unwrap();
        db.execute("INSERT INTO users (name, score) VALUES ('ann', 1.5), (NULL, 2)")
            .unwrap();

        assert_eq!(db.list_tables().unwrap(), vec!["users"]);
        assert_eq!(
            db.list_columns("users").unwrap(),
            vec![
                ("id".into(), "INTEGER".into()),
                ("name".into(), "TEXT".into()),
                ("score".into(), "REAL".into()),
            ]
        );
        let (cols, rows) = db
            .execute("SELECT name, score FROM users ORDER BY id")
            .unwrap();
        assert_eq!(cols, vec!["name", "score"]);
        assert_eq!(
            rows,
            vec![
                vec![Some("ann".into()), Some("1.5".into())],
                vec![None, Some("2.0".into())]
            ]
        );
        assert!(db.execute("SELECT nope").is_err());
    }
}
