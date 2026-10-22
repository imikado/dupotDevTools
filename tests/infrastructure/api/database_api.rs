use dupot_dev_tools::domain::contract::database_repository::DatabaseRepository;
use dupot_dev_tools::domain::entity::database::{ConnectionForm, DbKind, ForeignKey};
use dupot_dev_tools::infrastructure::api::database_api::DatabaseApi;

fn in_memory_db() -> DatabaseApi {
    let config = ConnectionForm {
        path: ":memory:".into(),
        ..Default::default()
    }
    .into_config(DbKind::Sqlite, 0)
    .unwrap();
    let mut db = DatabaseApi::default();
    db.connect(&config).unwrap();
    db
}

#[test]
fn not_connected_by_default() {
    let mut db = DatabaseApi::default();
    assert!(!db.is_connected());
    assert_eq!(db.kind(), None);
    assert_eq!(
        db.execute("SELECT 1").unwrap_err().message(),
        "Not connected."
    );
    assert!(db.list_tables().unwrap().is_empty());
}

#[test]
fn sqlite_tables_columns_and_rows() {
    let mut db = in_memory_db();
    assert_eq!(db.kind(), Some(DbKind::Sqlite));
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
            vec![None, Some("2.0".into())],
        ]
    );
}

#[test]
fn sql_errors_are_reported() {
    let mut db = in_memory_db();
    assert!(db.execute("SELECT nope").is_err());
}

#[test]
fn close_disconnects() {
    let mut db = in_memory_db();
    db.close();
    assert!(!db.is_connected());
}

#[test]
fn sqlite_foreign_keys() {
    let mut db = in_memory_db();
    db.execute("CREATE TABLE author (id INTEGER PRIMARY KEY)")
        .unwrap();
    db.execute(
        "CREATE TABLE book (id INTEGER PRIMARY KEY, author_id INTEGER REFERENCES author(id))",
    )
    .unwrap();
    assert_eq!(
        db.list_foreign_keys("book").unwrap(),
        [ForeignKey {
            column: "author_id".into(),
            ref_table: "author".into(),
            ref_column: "id".into(),
        }]
    );
    assert!(db.list_foreign_keys("author").unwrap().is_empty());
}
