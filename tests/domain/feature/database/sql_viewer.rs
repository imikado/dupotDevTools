use dupot_dev_tools::domain::entity::database::DbKind;
use dupot_dev_tools::domain::feature::database::sql_viewer::{
    CanvasJoin, CanvasTable, MAX_RESULT_ROWS, build_select, is_truncated, quote_ident, to_csv,
};

fn table(name: &str, columns: &[&str]) -> CanvasTable {
    CanvasTable {
        table: name.into(),
        selected_columns: columns.iter().map(|c| c.to_string()).collect(),
    }
}

fn join(left: usize, left_col: &str, right: usize, right_col: &str) -> CanvasJoin {
    CanvasJoin {
        left_table: left,
        left_column: left_col.into(),
        right_table: right,
        right_column: right_col.into(),
    }
}

#[test]
fn quote_ident_per_dialect() {
    assert_eq!(quote_ident(DbKind::Mysql, "a`b"), "`a``b`");
    assert_eq!(quote_ident(DbKind::Postgresql, "a\"b"), "\"a\"\"b\"");
    assert_eq!(quote_ident(DbKind::Sqlite, "users"), "\"users\"");
}

#[test]
fn single_table() {
    let sql = build_select(DbKind::Sqlite, &[table("users", &["id", "name"])], &[], 200).unwrap();
    assert_eq!(
        sql,
        "SELECT t1.\"id\" AS \"t1_id\", t1.\"name\" AS \"t1_name\" FROM \"users\" AS t1 LIMIT 200"
    );
}

#[test]
fn joins_are_ordered_from_the_first_table() {
    // users(t1) <- orders(t2) <- items(t3), joins given out of order.
    let tables = [
        table("users", &["name"]),
        table("orders", &[]),
        table("items", &["sku"]),
    ];
    let joins = [join(2, "order_id", 1, "id"), join(1, "user_id", 0, "id")];
    let sql = build_select(DbKind::Mysql, &tables, &joins, 10).unwrap();
    assert_eq!(
        sql,
        "SELECT t1.`name` AS \"t1_name\", t3.`sku` AS \"t3_sku\" FROM `users` AS t1 \
         INNER JOIN `orders` AS t2 ON t1.`id` = t2.`user_id` \
         INNER JOIN `items` AS t3 ON t2.`id` = t3.`order_id` LIMIT 10"
    );
}

#[test]
fn no_table_is_an_error() {
    let err = build_select(DbKind::Sqlite, &[], &[], 200).unwrap_err();
    assert_eq!(err.message(), "Add at least one table to the canvas first.");
}

#[test]
fn no_selected_column_is_an_error() {
    let err = build_select(DbKind::Sqlite, &[table("users", &[])], &[], 200).unwrap_err();
    assert_eq!(err.message(), "Check at least one column to select.");
}

#[test]
fn unlinked_tables_are_reported() {
    let tables = [
        table("users", &["id"]),
        table("logs", &["id"]),
        table("tags", &[]),
    ];
    let err = build_select(DbKind::Sqlite, &tables, &[], 200).unwrap_err();
    assert_eq!(err.message(), "Link a column to connect: logs, tags");
}

#[test]
fn truncation_matches_the_row_cap() {
    assert!(is_truncated(MAX_RESULT_ROWS));
    assert!(!is_truncated(MAX_RESULT_ROWS - 1));
}

#[test]
fn csv_has_header_and_empty_nulls() {
    let columns = vec!["id".to_string(), "note".to_string()];
    let rows = vec![
        vec![Some("1".into()), Some("a, b".into())],
        vec![Some("2".into()), None],
    ];
    assert_eq!(
        to_csv(&columns, &rows).unwrap(),
        "id,note\n1,\"a, b\"\n2,\n"
    );
}
