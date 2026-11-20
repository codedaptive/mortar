//! A config key whose value is a non-negative integer.

use oplog::JsonValue;

/// The number in `slot`; refused, naming `key`, when the key is missing or the value is not a number.
pub(crate) fn take_number(slot: Option<JsonValue>, key: &str) -> Result<u64, String> {
    match slot {
        Some(JsonValue::Number(n)) => Ok(n),
        Some(_) => Err(format!("{key} is not a number")),
        None => Err(format!("missing key {key:?}")),
    }
}
