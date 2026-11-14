//! A JSON value of the subset the config uses.
//! A bare type (S-7); parse_config_json_tests and the take_* tests exercise it.
//! A stopgap: oplog keeps its own JsonValue and parse_json crate-private, so the cli reads its config with
//! this layer over oplog::parse_json_string (S-51) until oplog exports parse_json and JsonValue (S-80,
//! WO-oplog-11), when this file and parse_config_json.rs go.

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
