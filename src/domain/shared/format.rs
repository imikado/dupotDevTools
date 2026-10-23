/// One "label  value" line per row, values aligned two spaces after the
/// longest label.
pub fn rows<L: AsRef<str>, V: AsRef<str>>(rows: &[(L, V)]) -> String {
    let width = rows
        .iter()
        .map(|(l, _)| l.as_ref().chars().count())
        .max()
        .unwrap_or(0);
    rows.iter()
        .map(|(l, v)| {
            format!("{:<width$}  {}", l.as_ref(), v.as_ref())
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `value` with at most `decimals` decimals, without trailing zeros; tiny
/// non-zero values that would round to 0 use scientific notation.
pub fn number(value: f64, decimals: usize) -> String {
    let fixed = format!("{value:.decimals$}");
    let fixed = if fixed.contains('.') {
        fixed
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        fixed
    };
    if value != 0.0 && (fixed == "0" || fixed == "-0") {
        return format!("{value:.3e}");
    }
    if fixed == "-0" { "0".into() } else { fixed }
}
