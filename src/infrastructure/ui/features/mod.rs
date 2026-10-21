//! Every tool is split in two mirrored modules:
//! - `domain::feature::<section>::<name>`: its `ENTITY` and pure logic;
//! - `infrastructure::ui::features::<section>::<name>/`: its view (`mod.rs`)
//!   and `icon.png`.
//!
//! and listed once in `registry()`.

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::feature;

use super::feature_contract::FeatureContract;

pub mod convert;
pub mod database;
pub mod encoding;
pub mod formating;
pub mod parsing;
pub mod string;

/// A tool as shown by the UI: its domain entity, icon and view factory.
#[derive(Clone)]
pub struct RegisteredFeature {
    pub entity: FeatureEntity,
    pub icon: Option<&'static [u8]>,
    pub create: fn() -> Box<dyn FeatureContract>,
}

macro_rules! feature {
    ($section:ident :: $name:ident) => {
        RegisteredFeature {
            entity: feature::$section::$name::ENTITY,
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

pub fn registry() -> Vec<RegisteredFeature> {
    vec![
        feature!(convert::html_to_text),
        feature!(convert::timestamp),
        feature!(convert::yaml_json),
        feature!(database::sql_viewer),
        feature!(encoding::base64_encoding),
        feature!(encoding::html_encoding),
        feature!(encoding::jwt_decode),
        feature!(encoding::url_encoding),
        feature!(formating::json_format),
        feature!(formating::sql_format),
        feature!(formating::xml_format),
        feature!(parsing::cron_parser),
        feature!(string::hash),
        feature!(string::regex_tester),
        feature!(string::text_diff),
        feature!(string::uuid_generator),
    ]
}
