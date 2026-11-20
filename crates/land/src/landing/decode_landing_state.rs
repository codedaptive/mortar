//! decode_landing_state: reads the text of landing.json back into a LandingState.
//! No sibling unit test: every test of it runs oplog's parse_json_string. Covered by tests/landing_zit
//! (landing.rs, a_second_landing_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged: the second landing
//! answers Busy with the change ID read back from the state, and the repeat answers Landed at the commit it names); its
//! unit cases are repo integration tests (RITs): tests/land_rit/state_codec.rs covers them.
//! Why a strict reader of one layout, not a general JSON parser: only encode_landing_state writes this file, so text
//! in any other layout is not a state land wrote, and land, back_out and publish must not act on it. Strings are read
//! by oplog's parse_json_string, which takes every JSON escape.

use crate::landing::LandingState;
use gitio::Oid;
use oplog::{parse_json_string, ChangeId, JsonError};

/// Reads exactly the layout encode_landing_state writes: one JSON object with no whitespace, its keys in
/// encode_landing_state's order, and at most one trailing newline. Strings may hold any JSON escape; change_id must
/// be a change ID and the ids object ids. Anything else is refused with the reason and the byte it was found at.
pub(crate) fn decode_landing_state(text: &str) -> Result<LandingState, String> {
    let body = text.strip_suffix('\n').unwrap_or(text);
    let (change_id, at) = change_id(body, literal(body, 0, "{\"change_id\":")?)?;
    let (zone, at) = string(body, key(body, at, "zone")?)?;
    let (squash, at) = id(body, key(body, at, "squash")?)?;
    let (origin_at_start, at) = id(body, key(body, at, "origin_at_start")?)?;
    let (integration, at) = id(body, key(body, at, "integration")?)?;
    if body.get(at..) != Some("}") {
        return Err(format!("expected the closing brace and the end of the text at byte {at}"));
    }
    Ok(LandingState { change_id, zone, squash, origin_at_start, integration })
}

/// The index after `lit`, which must start at byte `at`.
fn literal(body: &str, at: usize, lit: &str) -> Result<usize, String> {
    match body.get(at..) {
        Some(rest) if rest.starts_with(lit) => Ok(at + lit.len()),
        _ => Err(format!("expected {lit} at byte {at}")),
    }
}

/// The index after `,"<name>":`, the separator and key encode_landing_state writes before each value but the first.
fn key(body: &str, at: usize, name: &str) -> Result<usize, String> {
    literal(body, at, &format!(",\"{name}\":"))
}

/// The string whose opening quote is at byte `at`, and the index after its closing quote.
fn string(body: &str, at: usize) -> Result<(String, usize), String> {
    parse_json_string(body, literal(body, at, "\"")?).map_err(|error| match error {
        JsonError::EndOfInput => {
            format!("the landing state's text could not be read: it ends inside the string starting at byte {at}")
        }
        JsonError::Invalid(why) => format!("the landing state's text could not be read: {why}"),
    })
}

fn id(body: &str, at: usize) -> Result<(Oid, usize), String> {
    let (hex, next) = string(body, at)?;
    let oid = Oid::parse(&hex).map_err(|error| format!("at byte {at}: {error}"))?;
    Ok((oid, next))
}

fn change_id(body: &str, at: usize) -> Result<(ChangeId, usize), String> {
    let (text, next) = string(body, at)?;
    let id = ChangeId::parse(&text).map_err(|error| format!("at byte {at}: {error}"))?;
    Ok((id, next))
}
