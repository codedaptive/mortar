//! A config key whose value is a name: a non-empty string.

use super::ConfigJsonValue;

/// The string in `slot`; refused, naming `key`, when the key is missing, the value is not a string, or the
/// string is empty (a branch or remote has a name; Mortar holds no default).
pub(crate) fn take_name(slot: Option<ConfigJsonValue>, key: &str) -> Result<String, String> {
    match slot {
        Some(ConfigJsonValue::String(s)) if s.is_empty() => Err(format!("{key} is empty; it must name something")),
        Some(ConfigJsonValue::String(s)) => Ok(s),
        Some(_) => Err(format!("{key} is not a string")),
        None => Err(format!("missing key {key:?}")),
    }
}
