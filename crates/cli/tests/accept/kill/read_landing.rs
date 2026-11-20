//! Reads the text of `<common>/mortar/landing.json`: one JSON object with the string members change_id, zone,
//! squash, origin_at_start and integration (crates/land/src/landing/encode_landing_state.rs).

use super::LandingSeen;
use crate::json::{parse_json, JsonValue};

/// The landing the text holds; an error naming the first member that is missing or not a string, or the parse
/// error.
pub fn read_landing(text: &str) -> Result<LandingSeen, String> {
    let state = parse_json(text)?;
    Ok(LandingSeen {
        change_id: member(&state, "change_id")?,
        zone: member(&state, "zone")?,
        squash: member(&state, "squash")?,
        origin_at_start: member(&state, "origin_at_start")?,
        integration: member(&state, "integration")?,
    })
}

fn member(state: &JsonValue, name: &str) -> Result<String, String> {
    match state.field(name) {
        Some(JsonValue::Str(text)) => Ok(text.clone()),
        _ => Err(format!("landing.json has no string {name}")),
    }
}
