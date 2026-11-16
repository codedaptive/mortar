//! One parsed JSON value. Data only: parse_json_tests.rs and json_value_field_tests.rs cover how it is made and read.

/// A number keeps its text, so nothing is lost to a float. An object keeps its members in the order they came.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
