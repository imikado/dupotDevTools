use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Compiler, Ctx, Vars, data};
use jaq_json::Val;
use jaq_json::write::{Pp, write};
use serde_json::Value;
use serde_json_path::JsonPath;

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::shared::json::to_pretty_json;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "json_query",
    section: "parsing",
    title: "JSONPath / jq Tester",
    description: "Run JSONPath and jq expressions on JSON",
};

/// Results beyond this count are dropped (e.g. `repeat` or `range(1e9)`).
pub const MAX_RESULTS: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryLanguage {
    JsonPath,
    Jq,
}

impl QueryLanguage {
    pub const ALL: [QueryLanguage; 2] = [QueryLanguage::JsonPath, QueryLanguage::Jq];

    pub fn label(self) -> &'static str {
        match self {
            QueryLanguage::JsonPath => "JSONPath",
            QueryLanguage::Jq => "jq",
        }
    }

    pub fn placeholder(self) -> &'static str {
        match self {
            QueryLanguage::JsonPath => "JSONPath, e.g. $.store.book[?@.price < 10].title",
            QueryLanguage::Jq => "jq filter, e.g. .store.book[] | select(.price < 10) | .title",
        }
    }
}

pub const SAMPLE: &str = r#"{
    "store": {
        "book": [
            { "category": "reference", "author": "Nigel Rees", "title": "Sayings of the Century", "price": 8.95 },
            { "category": "fiction", "author": "Evelyn Waugh", "title": "Sword of Honour", "price": 12.99 },
            { "category": "fiction", "author": "Herman Melville", "title": "Moby Dick", "isbn": "0-553-21311-3", "price": 8.99 },
            { "category": "fiction", "author": "J. R. R. Tolkien", "title": "The Lord of the Rings", "isbn": "0-395-19395-8", "price": 22.99 }
        ],
        "bicycle": { "color": "red", "price": 399 }
    }
}"#;

/// JSONPath (RFC 9535) matches, as a pretty JSON array.
pub fn json_path(expr: &str, json: &str) -> DomainResult<String> {
    let path = JsonPath::parse(expr)?;
    let value: Value = serde_json::from_str(json)?;
    let matches: Vec<&Value> = path.query(&value).all();
    to_pretty_json(&matches)
}

/// Byte offset of `part` (a slice of `code`) in `code`.
fn offset(code: &str, part: &str) -> usize {
    (part.as_ptr() as usize).saturating_sub(code.as_ptr() as usize)
}

fn load_error(code: &str, error: jaq_core::load::Error<&str>) -> String {
    use jaq_core::load::Error;
    match error {
        Error::Io(errors) => errors
            .into_iter()
            .map(|(_, e)| e)
            .collect::<Vec<_>>()
            .join("\n"),
        Error::Lex(errors) => errors
            .into_iter()
            .map(|(expect, at)| {
                format!(
                    "expected {} at position {}",
                    expect.as_str(),
                    offset(code, at)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Error::Parse(errors) => errors
            .into_iter()
            .map(|(expect, at)| {
                let found = if at.is_empty() {
                    "end of filter".into()
                } else {
                    format!("{at:?}")
                };
                format!(
                    "expected {}, found {found} at position {}",
                    expect.as_str(),
                    offset(code, at)
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn pretty(value: &Val) -> DomainResult<String> {
    let pp = Pp {
        indent: Some("    ".to_string()),
        sep_space: true,
        ..Default::default()
    };
    let mut out = Vec::new();
    write(&mut out, &pp, 0, value)?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}

/// jq filter outputs, one pretty JSON value after another; each JSON value of
/// the input (JSON lines allowed) is filtered in turn.
pub fn jq(filter: &str, json: &str) -> DomainResult<String> {
    let program = File {
        code: filter,
        path: (),
    };
    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());
    let funs = jaq_core::funs()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());
    let arena = Arena::default();
    let modules = Loader::new(defs).load(&arena, program).map_err(|errors| {
        DomainError::new(
            errors
                .into_iter()
                .map(|(_, e)| load_error(filter, e))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    })?;
    let compiled = Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|errors| {
            let undefined: Vec<String> = errors
                .into_iter()
                .flat_map(|(_, e)| e)
                .map(|(name, kind)| format!("undefined {kind:?}: {name}").to_lowercase())
                .collect();
            DomainError::new(undefined.join("\n"))
        })?;

    let mut outputs = Vec::new();
    for input in jaq_json::read::parse_many(json.as_bytes()) {
        let input = input.map_err(|e| DomainError::new(format!("Invalid JSON: {e}")))?;
        let ctx = Ctx::<data::JustLut<Val>>::new(&compiled.lut, Vars::new([]));
        for output in compiled.id.run((ctx, input)) {
            if outputs.len() >= MAX_RESULTS {
                break;
            }
            match output {
                Ok(value) => outputs.push(pretty(&value)?),
                Err(exn) => match exn.get_err() {
                    Ok(error) => return Err(DomainError::new(error.to_string())),
                    Err(_) => return Ok(outputs.join("\n")),
                },
            }
        }
    }
    Ok(outputs.join("\n"))
}

pub fn run(language: QueryLanguage, expr: &str, json: &str) -> DomainResult<String> {
    if expr.trim().is_empty() {
        return Err(DomainError::new(format!(
            "Enter a {} expression.",
            language.label()
        )));
    }
    match language {
        QueryLanguage::JsonPath => json_path(expr.trim(), json),
        QueryLanguage::Jq => jq(expr, json),
    }
}
