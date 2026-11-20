//! The config's `formatter` member.

use oplog::JsonValue;

use super::{take_members, take_number, take_strings, FormatterConfig};

/// null is None; otherwise exactly `{"command": [strings, at least one], "timeout_seconds": number}`.
pub(crate) fn decode_formatter(slot: Option<JsonValue>) -> Result<Option<FormatterConfig>, String> {
    let value = match slot {
        Some(JsonValue::Null) => return Ok(None),
        Some(value @ JsonValue::Object(_)) => value,
        Some(_) => return Err("formatter is neither null nor an object".to_string()),
        None => return Err("missing key \"formatter\"".to_string()),
    };
    let [command, timeout] = take_members(value, ["command", "timeout_seconds"], "formatter")?;
    let command = take_strings(command, "formatter.command")?;
    if command.is_empty() {
        return Err("formatter.command is empty; it must name the formatter's program".to_string());
    }
    let timeout_seconds = take_number(timeout, "formatter.timeout_seconds")?;
    Ok(Some(FormatterConfig { command, timeout_seconds }))
}
