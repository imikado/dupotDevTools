use std::collections::HashSet;

use crate::domain::entity::database::{Cell, DbKind};
use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "sql_viewer",
    section: "database",
    title: "SQL Viewer",
    description: "Browse database tables on an interactive canvas",
};

/// Row cap appended to generated queries.
pub const MAX_RESULT_ROWS: usize = 200;

/// A table placed on the canvas, with the columns checked for SELECT.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasTable {
    pub table: String,
    pub selected_columns: Vec<String>,
}

/// A join line between two canvas tables (indices into the table list).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasJoin {
    pub left_table: usize,
    pub left_column: String,
    pub right_table: usize,
    pub right_column: String,
}

/// Quotes an identifier for the database dialect (backticks for MySQL,
/// double quotes otherwise), doubling embedded quote characters.
pub fn quote_ident(kind: DbKind, name: &str) -> String {
    match kind {
        DbKind::Mysql => format!("`{}`", name.replace('`', "``")),
        _ => format!("\"{}\"", name.replace('"', "\"\"")),
    }
}

/// Builds `SELECT … FROM first_table INNER JOIN …  LIMIT n` from the canvas.
///
/// Tables are aliased `t1`, `t2`… in canvas order and selected columns as
/// `"t1_column"`. The first table is the FROM; joins are applied in an order
/// that only ever links a new table to one already joined, and every table
/// must be reachable through joins.
pub fn build_select(
    kind: DbKind,
    tables: &[CanvasTable],
    joins: &[CanvasJoin],
    limit: usize,
) -> DomainResult<String> {
    if tables.is_empty() {
        return Err(DomainError::new(
            "Add at least one table to the canvas first.",
        ));
    }
    let alias = |i: usize| format!("t{}", i + 1);
    let q = |name: &str| quote_ident(kind, name);

    let select_parts: Vec<String> = tables
        .iter()
        .enumerate()
        .flat_map(|(i, t)| {
            t.selected_columns.iter().map(move |col| {
                format!(
                    "{a}.{} AS \"{a}_{}\"",
                    quote_ident(kind, col),
                    col.replace('"', "\"\""),
                    a = alias(i)
                )
            })
        })
        .collect();
    if select_parts.is_empty() {
        return Err(DomainError::new("Check at least one column to select."));
    }

    let mut joined: HashSet<usize> = HashSet::from([0]);
    let mut remaining: Vec<&CanvasJoin> = joins.iter().collect();
    let mut join_clauses = Vec::new();
    let mut progress = true;
    while !remaining.is_empty() && progress {
        progress = false;
        remaining.retain(|j| {
            let (old_t, old_c, new_t, new_c) = match (
                joined.contains(&j.left_table),
                joined.contains(&j.right_table),
            ) {
                (true, false) => (j.left_table, &j.left_column, j.right_table, &j.right_column),
                (false, true) => (j.right_table, &j.right_column, j.left_table, &j.left_column),
                _ => return true,
            };
            join_clauses.push(format!(
                "INNER JOIN {} AS {} ON {}.{} = {}.{}",
                q(&tables[new_t].table),
                alias(new_t),
                alias(old_t),
                q(old_c),
                alias(new_t),
                q(new_c),
            ));
            joined.insert(new_t);
            progress = true;
            false
        });
    }

    let unreached: Vec<&str> = tables
        .iter()
        .enumerate()
        .filter(|(i, _)| !joined.contains(i))
        .map(|(_, t)| t.table.as_str())
        .collect();
    if !unreached.is_empty() {
        return Err(DomainError::new(format!(
            "Link a column to connect: {}",
            unreached.join(", ")
        )));
    }

    let mut sql = format!(
        "SELECT {} FROM {} AS {}",
        select_parts.join(", "),
        q(&tables[0].table),
        alias(0)
    );
    for clause in &join_clauses {
        sql.push(' ');
        sql.push_str(clause);
    }
    sql.push_str(&format!(" LIMIT {limit}"));
    Ok(sql)
}

/// Whether a result likely hit the generated query's row cap.
pub fn is_truncated(row_count: usize) -> bool {
    row_count == MAX_RESULT_ROWS
}

/// Serializes a query result as CSV (header row first, NULL as empty).
pub fn to_csv(columns: &[String], rows: &[Vec<Cell>]) -> DomainResult<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(columns)?;
    for row in rows {
        writer.write_record(row.iter().map(|v| v.as_deref().unwrap_or("")))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|e| DomainError::new(e.to_string()))?;
    Ok(String::from_utf8(bytes)?)
}
