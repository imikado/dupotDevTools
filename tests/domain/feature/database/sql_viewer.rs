use dupot_dev_tools::domain::entity::database::DbKind;
use dupot_dev_tools::domain::feature::database::sql_viewer::{
    CanvasJoin, CanvasTable, MAX_RESULT_ROWS, build_select, is_truncated, parse_select,
    quote_ident, to_csv,
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
    let sql = build_select(
        DbKind::Sqlite,
        &[table("users", &["id", "name"])],
        &[],
        &[],
        200,
    )
    .unwrap();
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
    let sql = build_select(DbKind::Mysql, &tables, &joins, &[], 10).unwrap();
    assert_eq!(
        sql,
        "SELECT t1.`name` AS \"t1_name\", t3.`sku` AS \"t3_sku\" FROM `users` AS t1 \
         INNER JOIN `orders` AS t2 ON t1.`id` = t2.`user_id` \
         INNER JOIN `items` AS t3 ON t2.`id` = t3.`order_id` LIMIT 10"
    );
}

#[test]
fn no_table_is_an_error() {
    let err = build_select(DbKind::Sqlite, &[], &[], &[], 200).unwrap_err();
    assert_eq!(err.message(), "Add at least one table to the canvas first.");
}

#[test]
fn no_selected_column_is_an_error() {
    let err = build_select(DbKind::Sqlite, &[table("users", &[])], &[], &[], 200).unwrap_err();
    assert_eq!(err.message(), "Check at least one column to select.");
}

#[test]
fn unlinked_tables_are_reported() {
    let tables = [
        table("users", &["id"]),
        table("logs", &["id"]),
        table("tags", &[]),
    ];
    let err = build_select(DbKind::Sqlite, &tables, &[], &[], 200).unwrap_err();
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

mod foreign_keys {
    use dupot_dev_tools::domain::entity::database::ForeignKey;
    use dupot_dev_tools::domain::feature::database::sql_viewer::{CanvasJoin, foreign_key_joins};

    fn fk(column: &str, ref_table: &str, ref_column: &str) -> ForeignKey {
        ForeignKey {
            column: column.into(),
            ref_table: ref_table.into(),
            ref_column: ref_column.into(),
        }
    }

    #[test]
    fn links_new_table_in_both_directions() {
        let book_fks = [fk("author_id", "author", "id")];
        let review_fks = [fk("book_id", "book", "id")];
        let tables: [(&str, &[ForeignKey]); 3] = [
            ("author", &[]),
            ("review", &review_fks),
            ("book", &book_fks),
        ];
        assert_eq!(
            foreign_key_joins(&tables, 2),
            [
                CanvasJoin {
                    left_table: 1,
                    left_column: "book_id".into(),
                    right_table: 2,
                    right_column: "id".into(),
                },
                CanvasJoin {
                    left_table: 2,
                    left_column: "author_id".into(),
                    right_table: 0,
                    right_column: "id".into(),
                },
            ]
        );
    }

    #[test]
    fn ignores_tables_not_on_the_canvas() {
        let book_fks = [fk("author_id", "author", "id")];
        let tables: [(&str, &[ForeignKey]); 1] = [("book", &book_fks)];
        assert!(foreign_key_joins(&tables, 0).is_empty());
    }
}

mod parsing {
    use dupot_dev_tools::domain::entity::database::DbKind;
    use dupot_dev_tools::domain::feature::database::sql_viewer::{
        CanvasJoin, CanvasTable, MAX_RESULT_ROWS, ParsedTable, build_select, parse_select,
    };

    fn join(l: usize, lc: &str, r: usize, rc: &str) -> CanvasJoin {
        CanvasJoin {
            left_table: l,
            left_column: lc.into(),
            right_table: r,
            right_column: rc.into(),
        }
    }

    #[test]
    fn reads_tables_columns_and_on_joins() {
        let parsed = parse_select(
            DbKind::Postgresql,
            "SELECT a.name, b.title FROM author a \
             LEFT JOIN book AS b ON (b.author_id = a.id AND b.price > 1) \
             WHERE a.country = 'France' ORDER BY a.name",
        )
        .unwrap();
        assert_eq!(
            parsed.tables,
            [
                ParsedTable {
                    table: "author".into(),
                    columns: Some(vec!["name".into()]),
                },
                ParsedTable {
                    table: "book".into(),
                    columns: Some(vec!["title".into()]),
                },
            ]
        );
        assert_eq!(parsed.joins, [join(1, "author_id", 0, "id")]);
    }

    #[test]
    fn reads_wildcards_using_and_where_joins() {
        let parsed = parse_select(
            DbKind::Sqlite,
            "SELECT author.*, title, x.y FROM author, book JOIN shelf USING (book_id) \
             WHERE book.author_id = author.id",
        )
        .unwrap();
        let names: Vec<_> = parsed.tables.iter().map(|t| t.table.as_str()).collect();
        assert_eq!(names, ["author", "book", "shelf"]);
        assert_eq!(parsed.tables[0].columns, None);
        assert_eq!(parsed.tables[1].columns, Some(vec![]));
        assert_eq!(parsed.unqualified_columns, ["title"]);
        assert_eq!(
            parsed.joins,
            [
                join(1, "book_id", 2, "book_id"),
                join(1, "author_id", 0, "id")
            ]
        );
    }

    #[test]
    fn round_trips_a_generated_query() {
        let tables = [
            CanvasTable {
                table: "author".into(),
                selected_columns: vec!["name".into()],
            },
            CanvasTable {
                table: "book".into(),
                selected_columns: vec!["title".into(), "price".into()],
            },
        ];
        let joins = [join(1, "author_id", 0, "id")];
        for kind in DbKind::ALL {
            let sql = build_select(kind, &tables, &joins, &[], MAX_RESULT_ROWS).unwrap();
            let parsed = parse_select(kind, &sql).unwrap();
            assert_eq!(
                parsed.tables[1].columns,
                Some(vec!["title".into(), "price".into()])
            );
            assert_eq!(parsed.joins, [join(0, "id", 1, "author_id")], "{kind:?}");
        }
    }

    #[test]
    fn refuses_what_cannot_be_drawn() {
        for sql in [
            "SELECT 1",
            "UPDATE book SET price = 1",
            "SELECT * FROM (SELECT * FROM book) b",
            "SELECT * FROM a UNION SELECT * FROM b",
            "SELEC oops",
        ] {
            assert!(parse_select(DbKind::Postgresql, sql).is_err(), "{sql}");
        }
    }
}

mod criteria {
    use dupot_dev_tools::domain::entity::database::DbKind;
    use dupot_dev_tools::domain::feature::database::sql_viewer::{
        CanvasJoin, CanvasTable, ColumnCriteria, MAX_RESULT_ROWS, SortDirection, build_select,
        filter_condition, parse_select,
    };

    #[test]
    fn filter_cells_become_conditions() {
        let col = "t1.\"c\"";
        assert_eq!(filter_condition(col, "> 8"), "t1.\"c\" > 8");
        assert_eq!(filter_condition(col, " like 'A%'"), "t1.\"c\" like 'A%'");
        assert_eq!(filter_condition(col, "IS NULL"), "t1.\"c\" IS NULL");
        assert_eq!(filter_condition(col, "IN (1, 2)"), "t1.\"c\" IN (1, 2)");
        assert_eq!(filter_condition(col, "12.5"), "t1.\"c\" = 12.5");
        assert_eq!(filter_condition(col, "'x'"), "t1.\"c\" = 'x'");
        assert_eq!(filter_condition(col, "O'Neil"), "t1.\"c\" = 'O''Neil'");
    }

    fn designer() -> (Vec<CanvasTable>, Vec<CanvasJoin>, Vec<ColumnCriteria>) {
        let tables = vec![
            CanvasTable {
                table: "author".into(),
                selected_columns: vec!["name".into()],
            },
            CanvasTable {
                table: "book".into(),
                selected_columns: vec!["title".into(), "price".into()],
            },
        ];
        let joins = vec![CanvasJoin {
            left_table: 1,
            left_column: "author_id".into(),
            right_table: 0,
            right_column: "id".into(),
        }];
        let criteria = vec![
            ColumnCriteria {
                table: 0,
                column: "name".into(),
                alias: "author".into(),
                sort: Some(SortDirection::Ascending),
                sort_order: 2,
                ..Default::default()
            },
            ColumnCriteria {
                table: 1,
                column: "price".into(),
                sort: Some(SortDirection::Descending),
                sort_order: 1,
                filter: "> 8".into(),
                or_filter: "IS NULL".into(),
                ..Default::default()
            },
            ColumnCriteria {
                table: 0,
                column: "country".into(),
                filter: "France".into(),
                ..Default::default()
            },
        ];
        (tables, joins, criteria)
    }

    #[test]
    fn builds_alias_where_and_order_by() {
        let (tables, joins, criteria) = designer();
        let sql = build_select(DbKind::Postgresql, &tables, &joins, &criteria, 50).unwrap();
        assert_eq!(
            sql,
            "SELECT t1.\"name\" AS \"author\", t2.\"title\" AS \"t2_title\", \
             t2.\"price\" AS \"t2_price\" FROM \"author\" AS t1 \
             INNER JOIN \"book\" AS t2 ON t1.\"id\" = t2.\"author_id\" \
             WHERE (t2.\"price\" > 8 AND t1.\"country\" = 'France') OR (t2.\"price\" IS NULL) \
             ORDER BY t2.\"price\" DESC, t1.\"name\" ASC LIMIT 50"
        );
    }

    #[test]
    fn criteria_round_trip_through_sql() {
        let (tables, joins, criteria) = designer();
        for kind in DbKind::ALL {
            let sql = build_select(kind, &tables, &joins, &criteria, MAX_RESULT_ROWS).unwrap();
            let parsed = parse_select(kind, &sql).unwrap();
            assert!(parsed.complete, "{kind:?}: {sql}");
            let find = |table: usize, column: &str| {
                parsed
                    .criteria
                    .iter()
                    .find(|c| c.table == table && c.column == column)
                    .cloned()
                    .unwrap_or_default()
            };
            let name = find(0, "name");
            assert_eq!(name.alias, "author");
            assert_eq!(
                (name.sort, name.sort_order),
                (Some(SortDirection::Ascending), 2)
            );
            let price = find(1, "price");
            assert_eq!(price.filter, "> 8");
            assert_eq!(price.or_filter, "IS NULL");
            assert_eq!(
                (price.sort, price.sort_order),
                (Some(SortDirection::Descending), 1)
            );
            assert_eq!(find(0, "country").filter, "= 'France'");
        }
    }

    #[test]
    fn unsupported_clauses_are_incomplete() {
        for sql in [
            "SELECT a.name FROM author a GROUP BY a.name",
            "SELECT DISTINCT a.name FROM author a",
            "SELECT count(a.id) FROM author a",
            "SELECT a.name FROM author a LEFT JOIN book b ON b.author_id = a.id",
            "SELECT a.name FROM author a WHERE a.id > 1 AND a.id < 5",
            "SELECT a.name FROM author a LIMIT 10",
        ] {
            let parsed = parse_select(DbKind::Postgresql, sql).unwrap();
            assert!(!parsed.complete, "{sql}");
        }
        let simple = parse_select(DbKind::Postgresql, "SELECT name FROM author WHERE id > 1");
        assert!(simple.unwrap().complete);
    }
}

#[test]
fn parse_select_keeps_both_sides_of_a_self_join() {
    let parsed = parse_select(
        DbKind::Postgresql,
        "SELECT t1.name, t2.name FROM users AS t1 INNER JOIN users AS t2 ON t2.parent_id = t1.id",
    )
    .unwrap();
    let tables: Vec<&str> = parsed.tables.iter().map(|t| t.table.as_str()).collect();
    assert_eq!(tables, ["users", "users"]);
    assert_eq!(parsed.joins, [join(1, "parent_id", 0, "id")]);
    assert!(parsed.complete);
}

#[test]
fn self_join_round_trips_through_build_and_parse() {
    let tables = [table("users", &["name"]), table("users", &["name"])];
    let joins = [join(1, "parent_id", 0, "id")];
    let sql = build_select(DbKind::Sqlite, &tables, &joins, &[], MAX_RESULT_ROWS).unwrap();
    let parsed = parse_select(DbKind::Sqlite, &sql).unwrap();
    assert_eq!(parsed.tables.len(), 2);
    assert_eq!(parsed.joins, [join(0, "id", 1, "parent_id")]);
    assert!(parsed.complete);
}
