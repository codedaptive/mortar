//! A config key whose value is a name: a non-empty string.

use oplog::JsonValue;

/// The string in `slot`; refused, naming `key`, when the key is missing, the value is not a string, or the
/// string is empty (a branch or remote has a name; Mortar holds no default).
pub(crate) fn take_name(slot: Option<JsonValue>, key: &str) -> Result<String, String> {
    match slot {
        Some(JsonValue::String(s)) if s.is_empty() => Err(format!("{key} is empty; it must name something")),
        Some(JsonValue::String(s)) => Ok(s),
        Some(_) => Err(format!("{key} is not a string")),
        None => Err(format!("missing key {key:?}")),
    }
}
