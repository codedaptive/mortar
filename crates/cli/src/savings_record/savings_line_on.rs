//! The user's switch for the "[mr] tokens saved" line.

use oplog::{parse_json, JsonValue};

/// False only when `config` (the text of the user's config.json) reads with oplog::parse_json as one object whose
/// last top-level "savings_line" member holding a boolean is false; members of nested values do not count, and
/// keys match after their escapes are decoded. No file (None), no such member, true, only non-boolean values, a top
/// level that is not an object, or text that does not parse: true, so a broken file never hides the line or fails
/// the verb. parse_json refuses negative numbers, fractions, exponents, integers over u64 and lone surrogates, so a
/// file holding one of those reads as unparsable and the line stays on.
pub fn savings_line_on(config: Option<&str>) -> bool {
    let Some(Ok(JsonValue::Object(members))) = config.map(parse_json) else {
        return true;
    };
    let last = members.iter().rev().find_map(|(key, value)| match value {
        JsonValue::Bool(b) if key == "savings_line" => Some(*b),
        _ => None,
    });
    last != Some(false)
}
