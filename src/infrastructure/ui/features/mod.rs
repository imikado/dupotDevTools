//! Every tool lives in `<section>/<name>/` (a `mod.rs` + `icon.png`) and is
//! listed once in `registry()`. A new section folder = a new sidebar entry.

use crate::domain::entity::FeatureEntity;

pub mod convert;
pub mod database;
pub mod encoding;
pub mod formating;
pub mod string;

macro_rules! feature {
    ($section:ident :: $name:ident) => {
        FeatureEntity {
            name: stringify!($name),
            title: $section::$name::TITLE,
            description: $section::$name::DESCRIPTION,
            section: stringify!($section),
            icon: Some(include_bytes!(concat!(
                stringify!($section),
                "/",
                stringify!($name),
                "/icon.png"
            ))),
            create: || Box::new($section::$name::Feature),
        }
    };
}

pub fn registry() -> Vec<FeatureEntity> {
    vec![
        feature!(convert::html_to_text),
        feature!(convert::yaml_json),
        feature!(database::sql_viewer),
        feature!(encoding::base64_encoding),
        feature!(encoding::html_encoding),
        feature!(encoding::jwt_decode),
        feature!(encoding::url_encoding),
        feature!(formating::json_format),
        feature!(formating::sql_format),
        feature!(formating::xml_format),
        feature!(string::hash),
    ]
}
