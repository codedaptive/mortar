//! encode_landing_state: a LandingState as the text of landing.json, in the one-landing form for a batch of one and
//! the batch form from the second member on. encode_landing_state_tests covers it. tests/landing_zit (landing.rs)
//! checks the text land writes, and tests/commit_files_zit (kills.rs) the text with the pending key.

use crate::landing::{LandingMember, LandingState};
use oplog::write_json_string;
use std::fmt::Write;

/// One JSON object with no whitespace and a trailing newline, then pending last only when there is one; change_id
/// and zone as JSON strings, the ids as hex strings. A state with no earlier member is written in the one-landing
/// form (keys change_id, zone, squash, origin_at_start, integration), byte for byte the layout written before
/// batches, so older readers see no change (Orc #5114). Two or more members are written in the batch form:
/// origin_at_start, then `landings`, an array of every member in landing order (each an object with keys
/// change_id, zone, squash, integration).
pub(crate) fn encode_landing_state(state: &LandingState) -> String {
    // Writing to a String cannot fail. Ids are hex, so they need no escaping.
    let mut out = if state.earlier.is_empty() { one_landing(state) } else { batch(state) };
    if let Some(pending) = &state.pending {
        let _ = write!(out, ",\"pending\":\"{pending}\"");
    }
    out.push_str("}\n");
    out
}

/// The one-landing form up to its integration, the object left open for pending.
fn one_landing(state: &LandingState) -> String {
    let mut out = String::from("{");
    names(&mut out, &state.newest);
    let _ = write!(
        out,
        ",\"squash\":\"{}\",\"origin_at_start\":\"{}\",\"integration\":\"{}\"",
        state.newest.squash, state.origin_at_start, state.newest.integration
    );
    out
}

/// The batch form up to the `landings` array's closing bracket, the object left open for pending.
fn batch(state: &LandingState) -> String {
    let mut out = format!("{{\"origin_at_start\":\"{}\",\"landings\":[", state.origin_at_start);
    for (index, member) in state.members().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('{');
        names(&mut out, member);
        let _ = write!(out, ",\"squash\":\"{}\",\"integration\":\"{}\"}}", member.squash, member.integration);
    }
    out.push(']');
    out
}

/// `"change_id":<json>,"zone":<json>`, the keys both forms start a member with.
fn names(out: &mut String, member: &LandingMember) {
    out.push_str("\"change_id\":");
    write_json_string(out, member.change_id.as_str());
    out.push_str(",\"zone\":");
    write_json_string(out, &member.zone);
}
