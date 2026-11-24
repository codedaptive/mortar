//! decode_landing_state: reads the text of landing.json back into a LandingState, in the batch form or the
//! one-landing form. decode_landing_state_tests covers it. Covered by tests/landing_zit (landing.rs,
//! a_second_squash_of_the_same_zone_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged: the second
//! squash answers Busy with the newest member's change ID, read back from the state, and the repeat
//! answers Landed at the commit it names), and
//! tests/commit_files_zit (kills.rs) reads a state with the pending key back through commit_files and publish.
//! Why a strict reader of two layouts, not a general JSON parser: only encode_landing_state writes this file (the
//! one-landing form is read as a batch of one, so a state in that layout, from a landing in flight across an upgrade,
//! finishes), so text in any other layout is not a state land wrote, and land, back_out and publish must not act on
//! it. Strings are read by oplog's parse_json_string, which takes every JSON escape.

use crate::landing::{LandingMember, LandingState};
use gitio::Oid;
use oplog::{parse_json_string, ChangeId, JsonError};

/// How the batch form starts; text starting any other way is read in the one-landing form.
const BATCH: &str = "{\"origin_at_start\":";

/// Reads exactly the layout encode_landing_state writes: one JSON object with no whitespace, origin_at_start, then
/// `landings` with at least one member (keys change_id, zone, squash, integration), the last member the newest; or
/// the one-landing form, its keys change_id, zone, squash, origin_at_start, integration, read as a batch of one.
/// Either form: pending last and only when present, and at most one trailing newline. Strings may hold any JSON
/// escape; change_id must be a change ID and the ids object ids. Anything else is refused with the reason and the
/// byte it was found at.
pub(crate) fn decode_landing_state(text: &str) -> Result<LandingState, String> {
    let body = text.strip_suffix('\n').unwrap_or(text);
    let (origin_at_start, earlier, newest, at) =
        if body.starts_with(BATCH) { batch(body)? } else { one_landing(body)? };
    let (pending, at) = if body.get(at..).is_some_and(|rest| rest.starts_with(",\"pending\":")) {
        let (pending, at) = id(body, key(body, at, "pending")?)?;
        (Some(pending), at)
    } else {
        (None, at)
    };
    if body.get(at..) != Some("}") {
        return Err(format!("expected the closing brace and the end of the text at byte {at}"));
    }
    Ok(LandingState { origin_at_start, earlier, newest, pending })
}

/// What both forms read before pending: origin_at_start, the earlier members, the newest member and the index after.
type Parsed = (Oid, Vec<LandingMember>, LandingMember, usize);

/// The batch form up to and including the `landings` array's closing bracket.
fn batch(body: &str) -> Result<Parsed, String> {
    let (origin_at_start, at) = id(body, BATCH.len())?;
    let at = key(body, at, "landings")?;
    let at = literal(body, at, "[")?;
    if body.get(at..).is_some_and(|rest| rest.starts_with(']')) {
        return Err(format!("expected a landing in the landings array at byte {at}: it is empty"));
    }
    let mut earlier = Vec::new();
    let (mut newest, mut at) = member(body, at)?;
    while body.get(at..).is_some_and(|rest| rest.starts_with(',')) {
        let (next, after) = member(body, at + 1)?;
        earlier.push(std::mem::replace(&mut newest, next));
        at = after;
    }
    Ok((origin_at_start, earlier, newest, literal(body, at, "]")?))
}

/// One member of the `landings` array, from its opening brace to after its closing brace.
fn member(body: &str, at: usize) -> Result<(LandingMember, usize), String> {
    let (change_id, at) = change_id(body, literal(body, at, "{\"change_id\":")?)?;
    let (zone, at) = string(body, key(body, at, "zone")?)?;
    let (squash, at) = id(body, key(body, at, "squash")?)?;
    let (integration, at) = id(body, key(body, at, "integration")?)?;
    Ok((LandingMember { change_id, zone, squash, integration }, literal(body, at, "}")?))
}

/// The one-landing form up to its integration, as a batch of one.
fn one_landing(body: &str) -> Result<Parsed, String> {
    let (change_id, at) = change_id(body, literal(body, 0, "{\"change_id\":")?)?;
    let (zone, at) = string(body, key(body, at, "zone")?)?;
    let (squash, at) = id(body, key(body, at, "squash")?)?;
    let (origin_at_start, at) = id(body, key(body, at, "origin_at_start")?)?;
    let (integration, at) = id(body, key(body, at, "integration")?)?;
    Ok((origin_at_start, Vec::new(), LandingMember { change_id, zone, squash, integration }, at))
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
