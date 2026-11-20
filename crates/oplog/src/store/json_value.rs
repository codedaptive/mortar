//! A parsed JSON value: null, booleans, unsigned integers, strings, arrays and objects.
//! A bare type; parse_json_tests and the decode tests exercise it.
//! Public: parse_json returns it.

/// No negative numbers and no fractions: parse_json refuses them.
/// An object keeps its keys in text order, duplicates included, so a caller decides what a repeat means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
