//! encode_landing_state: a LandingState as the text of landing.json.
//! No sibling unit test: every test of it runs oplog's write_json_string and gitio's Oid Display.
//! Covered by tests/landing_zit (landing.rs, a_squash_on_origin_fast_forwards_integration_to_the_squash: landing.json
//! holds exactly the hand-written state text); its unit cases are repo integration tests (RITs):
//! tests/land_rit/state_codec.rs covers them. tests/commit_files_zit (kills.rs) checks the text with the pending key.

use crate::landing::LandingState;
use oplog::write_json_string;
use std::fmt::Write;

/// One JSON object with no whitespace and a trailing newline, its keys in the order change_id, zone, squash,
/// origin_at_start, integration, then pending only when there is one; change_id and zone as JSON strings, the ids
/// as hex strings.
pub(crate) fn encode_landing_state(state: &LandingState) -> String {
    let mut out = String::from("{\"change_id\":");
    write_json_string(&mut out, state.change_id.as_str());
    out.push_str(",\"zone\":");
    write_json_string(&mut out, &state.zone);
    // Writing to a String cannot fail. Ids are hex, so they need no escaping.
    let _ = write!(
        out,
        ",\"squash\":\"{}\",\"origin_at_start\":\"{}\",\"integration\":\"{}\"",
        state.squash, state.origin_at_start, state.integration
    );
    if let Some(pending) = &state.pending {
        let _ = write!(out, ",\"pending\":\"{pending}\"");
    }
    out.push_str("}\n");
    out
}
