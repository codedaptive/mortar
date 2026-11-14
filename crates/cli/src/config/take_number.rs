//! A config key whose value is a non-negative integer.

use super::ConfigJsonValue;

/// The number in `slot`; refused, naming `key`, when the key is missing or the value is not a number.
pub(crate) fn take_number(slot: Option<ConfigJsonValue>, key: &str) -> Result<u64, String> {
    match slot {
        Some(ConfigJsonValue::Number(n)) => Ok(n),
        Some(_) => Err(format!("{key} is not a number")),
        None => Err(format!("missing key {key:?}")),
    }
}
