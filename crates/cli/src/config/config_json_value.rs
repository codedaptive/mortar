//! A JSON value of the subset the config uses.
//! A bare type; parse_config_json_tests and the take_* tests exercise it.
//! A stopgap: the cli reads its config with this layer over oplog::parse_json_string, though oplog already
//! exports parse_json and JsonValue; moving the config onto those retires this file and parse_config_json.rs.

/// No booleans and no fractions: the config holds none. Object members keep text order and repeats, so a
/// decoder can refuse a key given twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigJsonValue {
    Null,
    Number(u64),
    String(String),
    Array(Vec<ConfigJsonValue>),
    Object(Vec<(String, ConfigJsonValue)>),
}
