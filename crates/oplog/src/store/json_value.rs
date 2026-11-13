//! A parsed JSON value, limited to what an op record holds.
//! A bare type; parse_json_tests and the decode tests exercise it.

/// No booleans and no fractions: the record has none, so parse_json refuses them.
/// An object keeps its keys in text order, duplicates included, so the decoders can refuse a repeat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JsonValue {
    Null,
    Number(u64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
