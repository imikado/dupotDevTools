use std::collections::{HashMap, HashSet};

use sqlparser::ast::{
    BinaryOperator, Expr, GroupByExpr, Ident, JoinConstraint, JoinOperator, LimitClause,
    ObjectName, OrderByKind, OrderBySort, SelectItem, SelectItemQualifiedWildcardKind, SetExpr,
    Statement, TableFactor, Value,
};
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;

use crate::domain::entity::database::{Cell, DbKind, ForeignKey};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// One row of the criteria grid, as in SQL Server Management Studio's
/// query designer: how a canvas column is named, sorted and filtered.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColumnCriteria {
    /// Index into the canvas tables.
    pub table: usize,
    pub column: String,
    /// Output name; empty keeps the generated `tN_column`.
    pub alias: String,
    pub sort: Option<SortDirection>,
    /// Rank among sorted columns: lower sorts first.
    pub sort_order: u32,
    /// Condition on the column: `> 8`, `LIKE 'A%'`, `IS NULL`… or a bare
    /// value, compared with `=` (`France` → `= 'France'`).
    pub filter: String,
    /// Alternative condition: rows match every `filter` or every `or_filter`.
    pub or_filter: String,
}

const FILTER_OPERATORS: [&str; 7] = ["<>", "!=", "<=", ">=", "=", "<", ">"];
const FILTER_KEYWORDS: [&str; 8] = [
    "LIKE ",
    "NOT LIKE ",
    "ILIKE ",
    "NOT ILIKE ",
    "IN ",
    "IN(",
    "NOT IN",
    "IS ",
];

/// `column <condition>` for a criteria filter cell (see `ColumnCriteria::filter`).
pub fn filter_condition(column: &str, filter: &str) -> String {
    let filter = filter.trim();
    let upper = filter.to_uppercase();
    let has_operator = FILTER_OPERATORS.iter().any(|op| filter.starts_with(op))
        || FILTER_KEYWORDS.iter().any(|kw| upper.starts_with(kw))
        || upper.starts_with("BETWEEN ")
        || upper.starts_with("NOT BETWEEN ");
    if has_operator {
        return format!("{column} {filter}");
    }
    let is_literal = filter.parse::<f64>().is_ok()
        || (filter.len() >= 2 && filter.starts_with('\'') && filter.ends_with('\''))
        || upper == "NULL"
        || upper == "TRUE"
        || upper == "FALSE";
    if is_literal {
        format!("{column} = {filter}")
    } else {
        format!("{column} = '{}'", filter.replace('\'', "''"))
    }
}

/// Quotes an identifier for the database dialect (backticks for MySQL,
/// double quotes otherwise), doubling embedded quote characters.
pub fn quote_ident(kind: DbKind, name: &str) -> String {
    match kind {
        DbKind::Mysql => format!("`{}`", name.replace('`', "``")),
        _ => format!("\"{}\"", name.replace('"', "\"\"")),
    }
}

/// Builds `SELECT … FROM first_table INNER JOIN … WHERE … ORDER BY … LIMIT n`
/// from the canvas and its criteria.
///
/// Tables are aliased `t1`, `t2`… in canvas order and selected columns as
/// `"t1_column"` unless the criteria name them. The first table is the FROM;
/// joins are applied in an order that only ever links a new table to one
/// already joined, and every table must be reachable through joins.
pub fn build_select(
    kind: DbKind,
    tables: &[CanvasTable],
    joins: &[CanvasJoin],
    criteria: &[ColumnCriteria],
    limit: usize,
) -> DomainResult<String> {
    if tables.is_empty() {
        return Err(DomainError::new(
            "Add at least one table to the canvas first.",
        ));
    }
    let alias = |i: usize| format!("t{}", i + 1);
    let q = |name: &str| quote_ident(kind, name);
    let column_ref = |table: usize, column: &str| format!("{}.{}", alias(table), q(column));
    let criteria_of = |table: usize, column: &str| {
        criteria
            .iter()
            .find(|c| c.table == table && c.column == column)
    };

    let select_parts: Vec<String> = tables
        .iter()
        .enumerate()
        .flat_map(|(i, t)| {
            t.selected_columns.iter().map(move |col| {
                let name = match criteria_of(i, col) {
                    Some(c) if !c.alias.trim().is_empty() => c.alias.trim().to_string(),
                    _ => format!("{}_{col}", alias(i)),
                };
                format!(
                    "{} AS \"{}\"",
                    column_ref(i, col),
                    name.replace('"', "\"\"")
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

    let group = |cell: fn(&ColumnCriteria) -> &str| {
        criteria
            .iter()
            .filter(|c| c.table < tables.len() && !cell(c).trim().is_empty())
            .map(|c| filter_condition(&column_ref(c.table, &c.column), cell(c)))
            .collect::<Vec<_>>()
            .join(" AND ")
    };
    let groups: Vec<String> = [group(|c| &c.filter), group(|c| &c.or_filter)]
        .into_iter()
        .filter(|g| !g.is_empty())
        .collect();
    match groups.as_slice() {
        [] => {}
        [only] => sql.push_str(&format!(" WHERE {only}")),
        _ => sql.push_str(&format!(" WHERE ({}) OR ({})", groups[0], groups[1])),
    }

    let mut sorted: Vec<&ColumnCriteria> = criteria
        .iter()
        .filter(|c| c.sort.is_some() && c.table < tables.len())
        .collect();
    sorted.sort_by_key(|c| c.sort_order);
    if !sorted.is_empty() {
        let keys: Vec<String> = sorted
            .iter()
            .map(|c| {
                let direction = match c.sort {
                    Some(SortDirection::Descending) => "DESC",
                    _ => "ASC",
                };
                format!("{} {direction}", column_ref(c.table, &c.column))
            })
            .collect();
        sql.push_str(&format!(" ORDER BY {}", keys.join(", ")));
    }
    sql.push_str(&format!(" LIMIT {limit}"));
    Ok(sql)
}

/// Joins implied by foreign keys between `tables[new]` and the other tables,
/// in both directions. Each entry is a table name and its foreign keys.
pub fn foreign_key_joins(tables: &[(&str, &[ForeignKey])], new: usize) -> Vec<CanvasJoin> {
    let new_name = tables[new].0;
    let mut joins = Vec::new();
    for (i, (name, fks)) in tables.iter().enumerate() {
        if i == new {
            for fk in *fks {
                for (j, (other, _)) in tables.iter().enumerate() {
                    if j != new && *other == fk.ref_table {
                        joins.push(CanvasJoin {
                            left_table: new,
                            left_column: fk.column.clone(),
                            right_table: j,
                            right_column: fk.ref_column.clone(),
                        });
                    }
                }
            }
        } else if *name != new_name {
            for fk in fks.iter().filter(|fk| fk.ref_table == new_name) {
                joins.push(CanvasJoin {
                    left_table: i,
                    left_column: fk.column.clone(),
                    right_table: new,
                    right_column: fk.ref_column.clone(),
                });
            }
        }
    }
    joins
}

/// A table read from a SELECT and the columns it projects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedTable {
    pub table: String,
    /// `None` for every column (`*` or `alias.*`).
    pub columns: Option<Vec<String>>,
}

/// What of a SELECT can be drawn on the canvas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParsedSelect {
    /// Each table occurrence (a self-join lists it twice), in FROM order.
    pub tables: Vec<ParsedTable>,
    /// Column equalities between two tables, from `ON`, `USING` and `WHERE`.
    pub joins: Vec<CanvasJoin>,
    /// Columns selected without a table qualifier.
    pub unqualified_columns: Vec<String>,
    /// Aliases, filters and sorts, with `table` indexing `tables`.
    pub criteria: Vec<ColumnCriteria>,
    /// True when the canvas and criteria hold the whole query, so building
    /// it back from them loses nothing (bar formatting and the row cap).
    pub complete: bool,
}

/// Reads a single `SELECT … FROM table [JOIN …]` query. Clauses the canvas
/// can't show are skipped and clear `complete`; subqueries and set
/// operations are refused.
pub fn parse_select(kind: DbKind, sql: &str) -> DomainResult<ParsedSelect> {
    let dialect: Box<dyn Dialect> = match kind {
        DbKind::Sqlite => Box::new(SQLiteDialect {}),
        DbKind::Postgresql => Box::new(PostgreSqlDialect {}),
        DbKind::Mysql => Box::new(MySqlDialect {}),
    };
    let statements =
        Parser::parse_sql(dialect.as_ref(), sql).map_err(|e| DomainError::new(e.to_string()))?;
    let not_drawable = || DomainError::new("Only a single SELECT on tables can be drawn.");
    let [Statement::Query(query)] = statements.as_slice() else {
        return Err(not_drawable());
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Err(not_drawable());
    };

    let mut reader = Reader {
        parsed: ParsedSelect {
            complete: true,
            ..Default::default()
        },
        names: HashMap::new(),
    };
    let parsed_limit_ok = match &query.limit_clause {
        None => true,
        Some(LimitClause::LimitOffset {
            limit: Some(Expr::Value(v)),
            offset: None,
            limit_by,
        }) => limit_by.is_empty() && v.value == Value::Number(MAX_RESULT_ROWS.to_string(), false),
        Some(_) => false,
    };
    let has_group_by = match &select.group_by {
        GroupByExpr::Expressions(exprs, modifiers) => !exprs.is_empty() || !modifiers.is_empty(),
        GroupByExpr::All(_) => true,
    };
    if !parsed_limit_ok
        || has_group_by
        || query.with.is_some()
        || select.distinct.is_some()
        || select.top.is_some()
        || select.having.is_some()
    {
        reader.parsed.complete = false;
    }

    let mut on_conditions: Vec<&Expr> = Vec::new();
    let mut using: Vec<(usize, usize, String)> = Vec::new();
    for from in &select.from {
        let mut previous = reader.add_table(&from.relation).ok_or_else(not_drawable)?;
        for join in &from.joins {
            let index = reader.add_table(&join.relation).ok_or_else(not_drawable)?;
            if !matches!(
                join.join_operator,
                JoinOperator::Join(_) | JoinOperator::Inner(_)
            ) {
                // Drawn, but rebuilt as an INNER JOIN.
                reader.parsed.complete = false;
            }
            match join_constraint(&join.join_operator) {
                Some(JoinConstraint::On(expr)) => on_conditions.push(expr),
                Some(JoinConstraint::Using(columns)) => {
                    for column in columns.iter().filter_map(last_ident) {
                        using.push((previous, index, column));
                    }
                }
                _ => {}
            }
            previous = index;
        }
    }
    if reader.parsed.tables.is_empty() {
        return Err(not_drawable());
    }
    if select.from.len() > 1 {
        // `FROM a, b` is rebuilt with INNER JOINs.
        reader.parsed.complete = false;
    }

    for item in &select.projection {
        reader.read_projection(item);
    }

    for expr in on_conditions {
        for condition in split(expr, &BinaryOperator::And) {
            if !reader.read_join(condition) {
                reader.parsed.complete = false;
            }
        }
    }
    for (l, r, column) in using {
        reader.push_join((l, column.clone()), (r, column));
    }

    if let Some(selection) = &select.selection {
        let groups = split(selection, &BinaryOperator::Or);
        if groups.len() > 2 {
            reader.parsed.complete = false;
        }
        for (group, expr) in groups.iter().take(2).enumerate() {
            for condition in split(expr, &BinaryOperator::And) {
                let read = (groups.len() == 1 && reader.read_join(condition))
                    || reader.read_filter(condition, group);
                if !read {
                    reader.parsed.complete = false;
                }
            }
        }
    }

    if let Some(order_by) = &query.order_by {
        let OrderByKind::Expressions(keys) = &order_by.kind else {
            return Err(not_drawable());
        };
        for (rank, key) in keys.iter().enumerate() {
            let direction = match key.options.sort {
                None | Some(OrderBySort::Asc) => Some(SortDirection::Ascending),
                Some(OrderBySort::Desc) => Some(SortDirection::Descending),
                Some(OrderBySort::Using(_)) => None,
            };
            match (reader.column_ref(&key.expr), direction) {
                (Some((table, column)), Some(direction))
                    if key.options.nulls_first.is_none() && key.with_fill.is_none() =>
                {
                    let criteria = reader.criteria(table, &column);
                    criteria.sort = Some(direction);
                    criteria.sort_order = rank as u32 + 1;
                }
                _ => reader.parsed.complete = false,
            }
        }
    }
    Ok(reader.parsed)
}

struct Reader {
    parsed: ParsedSelect,
    /// Lowercased table name or alias → index in `parsed.tables`.
    names: HashMap<String, usize>,
}

impl Reader {
    fn add_table(&mut self, factor: &TableFactor) -> Option<usize> {
        let TableFactor::Table { name, alias, .. } = factor else {
            return None;
        };
        let table = last_ident(name)?;
        // Each occurrence is its own table, so a self-join keeps both sides.
        self.parsed.tables.push(ParsedTable {
            table: table.clone(),
            columns: Some(vec![]),
        });
        let index = self.parsed.tables.len() - 1;
        self.names.entry(table.to_lowercase()).or_insert(index);
        if let Some(alias) = alias {
            self.names.insert(alias.name.value.to_lowercase(), index);
        }
        Some(index)
    }

    /// `(table index, column)` of `qualifier.column`, or of a bare column
    /// when the query has a single table.
    fn column_ref(&self, expr: &Expr) -> Option<(usize, String)> {
        match expr {
            Expr::CompoundIdentifier(idents) => self.qualified(idents),
            Expr::Identifier(ident) if self.parsed.tables.len() == 1 => {
                Some((0, ident.value.clone()))
            }
            Expr::Nested(inner) => self.column_ref(inner),
            _ => None,
        }
    }

    fn qualified(&self, idents: &[Ident]) -> Option<(usize, String)> {
        match idents {
            [.., table, column] => self
                .names
                .get(&table.value.to_lowercase())
                .map(|&i| (i, column.value.clone())),
            _ => None,
        }
    }

    fn criteria(&mut self, table: usize, column: &str) -> &mut ColumnCriteria {
        let criteria = &mut self.parsed.criteria;
        let i = match criteria
            .iter()
            .position(|c| c.table == table && c.column == column)
        {
            Some(i) => i,
            None => {
                criteria.push(ColumnCriteria {
                    table,
                    column: column.to_string(),
                    ..Default::default()
                });
                criteria.len() - 1
            }
        };
        &mut criteria[i]
    }

    fn select_column(&mut self, table: usize, column: &str) {
        if let Some(columns) = &mut self.parsed.tables[table].columns
            && !columns.iter().any(|c| c == column)
        {
            columns.push(column.to_string());
        }
    }

    fn read_projection(&mut self, item: &SelectItem) {
        match item {
            SelectItem::Wildcard(_) => {
                for table in &mut self.parsed.tables {
                    table.columns = None;
                }
            }
            SelectItem::QualifiedWildcard(SelectItemQualifiedWildcardKind::ObjectName(name), _) => {
                let qualifier = last_ident(name).unwrap_or_default().to_lowercase();
                match self.names.get(&qualifier) {
                    Some(&i) => self.parsed.tables[i].columns = None,
                    None => self.parsed.complete = false,
                }
            }
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                let alias = match item {
                    SelectItem::ExprWithAlias { alias, .. } => Some(&alias.value),
                    _ => None,
                };
                match (expr, self.column_ref(expr)) {
                    (_, Some((table, column))) => {
                        self.select_column(table, &column);
                        // `t1.name AS "t1_name"` is what `build_select` writes anyway.
                        let qualifier = match expr {
                            Expr::CompoundIdentifier(idents) if idents.len() >= 2 => {
                                idents[idents.len() - 2].value.clone()
                            }
                            _ => String::new(),
                        };
                        if let Some(alias) = alias
                            && *alias != format!("{qualifier}_{column}")
                        {
                            self.criteria(table, &column).alias = alias.clone();
                        }
                    }
                    (Expr::Identifier(ident), None) => {
                        self.parsed.unqualified_columns.push(ident.value.clone());
                        self.parsed.complete = false;
                    }
                    _ => self.parsed.complete = false,
                }
            }
            _ => self.parsed.complete = false,
        }
    }

    fn push_join(&mut self, (lt, lc): (usize, String), (rt, rc): (usize, String)) -> bool {
        if lt == rt {
            return false;
        }
        let join = CanvasJoin {
            left_table: lt,
            left_column: lc,
            right_table: rt,
            right_column: rc,
        };
        if !self.parsed.joins.contains(&join) {
            self.parsed.joins.push(join);
        }
        true
    }

    /// Reads `a.x = b.y` between two tables as a join.
    fn read_join(&mut self, condition: &Expr) -> bool {
        let Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } = condition
        else {
            return false;
        };
        match (self.column_ref(left), self.column_ref(right)) {
            (Some(l), Some(r)) => self.push_join(l, r),
            _ => false,
        }
    }

    /// Reads `column <condition>` into the criteria's filter (`group` 0) or
    /// or-filter (`group` 1). A second condition on one cell isn't read.
    fn read_filter(&mut self, condition: &Expr, group: usize) -> bool {
        let column_expr = match condition {
            Expr::BinaryOp { left, op, right } => {
                let comparison = matches!(
                    op,
                    BinaryOperator::Eq
                        | BinaryOperator::NotEq
                        | BinaryOperator::Lt
                        | BinaryOperator::LtEq
                        | BinaryOperator::Gt
                        | BinaryOperator::GtEq
                );
                if !comparison || self.column_ref(right).is_some() {
                    return false;
                }
                left
            }
            Expr::Like { expr, .. }
            | Expr::ILike { expr, .. }
            | Expr::InList { expr, .. }
            | Expr::Between { expr, .. }
            | Expr::IsNull(expr)
            | Expr::IsNotNull(expr) => expr,
            _ => return false,
        };
        let Some((table, column)) = self.column_ref(column_expr) else {
            return false;
        };
        let full = condition.to_string();
        let Some(text) = full.strip_prefix(&column_expr.to_string()) else {
            return false;
        };
        let text = text.trim().to_string();
        let criteria = self.criteria(table, &column);
        let cell = if group == 0 {
            &mut criteria.filter
        } else {
            &mut criteria.or_filter
        };
        if !cell.is_empty() {
            return false;
        }
        *cell = text;
        true
    }
}

/// The operands of a chain of `op` (`a AND b AND c` → `[a, b, c]`), looking
/// through parentheses.
fn split<'a>(expr: &'a Expr, op: &BinaryOperator) -> Vec<&'a Expr> {
    match expr {
        Expr::Nested(inner) => split(inner, op),
        Expr::BinaryOp {
            left,
            op: found,
            right,
        } if found == op => {
            let mut parts = split(left, op);
            parts.extend(split(right, op));
            parts
        }
        _ => vec![expr],
    }
}

fn last_ident(name: &ObjectName) -> Option<String> {
    name.0.last()?.as_ident().map(|i| i.value.clone())
}

fn join_constraint(op: &JoinOperator) -> Option<&JoinConstraint> {
    match op {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c) => Some(c),
        _ => None,
    }
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
