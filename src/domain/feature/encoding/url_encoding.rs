use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};

use crate::domain::entity::feature_entity::FeatureEntity;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "url_encoding",
    section: "encoding",
    title: "URL Encoding",
    description: "URL percent-encode / decode",
};

/// RFC 3986 unreserved characters stay as-is, everything else is encoded.
const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub fn encode(input: &str) -> String {
    utf8_percent_encode(input, ENCODE_SET).to_string()
}

/// Decodes `%XX` sequences; invalid UTF-8 becomes U+FFFD.
pub fn decode(input: &str) -> String {
    percent_decode_str(input).decode_utf8_lossy().into_owned()
}
