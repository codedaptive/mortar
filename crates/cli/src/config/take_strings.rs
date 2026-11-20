//! A config key whose value is an array of strings.

use oplog::JsonValue;

/// The strings in `slot`, in order (empty strings kept); refused, naming `key`, when the key is missing,
/// the value is not an array, or an item is not a string.
pub(crate) fn take_strings(slot: Option<JsonValue>, key: &str) -> Result<Vec<String>, String> {
    let items = match slot {
        Some(JsonValue::Array(items)) => items,
        Some(_) => return Err(format!("{key} is not an array of strings")),
        None => return Err(format!("missing key {key:?}")),
    };
    items
        .into_iter()
        .enumerate()
        .map(|(i, item)| match item {
            JsonValue::String(s) => Ok(s),
            _ => Err(format!("{key} item {i} is not a string")),
        })
        .collect()
}
