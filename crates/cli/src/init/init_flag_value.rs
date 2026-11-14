//! The value after one of init's flags.

/// `value` (the next word, whatever it is) as `flag`'s value; refused when there is none or it is empty.
pub(crate) fn init_flag_value(flag: &str, value: Option<&String>) -> Result<String, String> {
    match value {
        None => Err(format!("{flag} needs a value")),
        Some(value) if value.is_empty() => Err(format!("{flag} needs a non-empty value")),
        Some(value) => Ok(value.clone()),
    }
}
