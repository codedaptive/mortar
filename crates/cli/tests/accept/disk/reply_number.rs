//! Reads one whole number from an `mr ... --json` reply.

use crate::json::JsonValue;
use crate::stage::Reply;

/// The report's top-level member `name` as a u64. Err when it is missing, null or not a JSON number (naming the
/// reply's saw), or when the number's text is not a u64, such as a negative, a fraction or one too large.
pub fn reply_number(reply: &Reply, name: &str) -> Result<u64, String> {
    match reply.report.field(name) {
        Some(JsonValue::Number(text)) => text.parse().map_err(|_| format!("{name} is not a whole number: {text}")),
        _ => Err(format!("the reply has no number {name}: {}", reply.saw)),
    }
}
