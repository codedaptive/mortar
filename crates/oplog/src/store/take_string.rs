//! A required string member, from its object_fields slot.

use super::JsonValue;

/// The string in `slot`; refuses an absent key or a value that is not a string, naming the key.
pub(crate) fn take_string(slot: Option<JsonValue>, name: &str) -> Result<String, String> {
    match slot {
        Some(JsonValue::String(s)) => Ok(s),
        Some(_) => Err(format!("{name} is not a string")),
        None => Err(format!("missing key {name:?}")),
    }
}
