//! encode_join_state: a JoinState as the text of mortar-join.json. encode_join_state_tests covers the
//! layout, empty arrays, both flag values, a rebased tip, string escapes and the refusal of a non-UTF-8 path;
//! decode_join_state_tests covers reading a hand-written text, not this encoder's output.

use crate::join_state::JoinState;
use oplog::write_json_string;
use std::fmt::Write;
use std::path::PathBuf;

/// One JSON object with no whitespace and a trailing newline, its 10 keys in the order request_ref, request_tip,
/// onto, replayed_tip, commit, tree, remaining, files, files_written, rebased_to; ids as hex strings, remaining
/// and files as arrays, files_written as `true` or `false`, rebased_to as `null` or a hex string. A conflicted
/// path that is not UTF-8 cannot be written without loss, so the first such path is returned as the error.
pub(crate) fn encode_join_state(state: &JoinState) -> Result<String, PathBuf> {
    let mut out = String::from("{\"request_ref\":");
    write_json_string(&mut out, &state.request_ref);
    let ids = [
        ("request_tip", &state.request_tip),
        ("onto", &state.onto),
        ("replayed_tip", &state.replayed_tip),
        ("commit", &state.commit),
        ("tree", &state.tree),
    ];
    // Writing to a String cannot fail, here and below. Ids are hex, so they need no escaping.
    for (name, id) in ids {
        let _ = write!(out, ",\"{name}\":\"{id}\"");
    }
    out.push_str(",\"remaining\":[");
    for (i, id) in state.remaining.iter().enumerate() {
        let _ = write!(out, "{}\"{id}\"", if i > 0 { "," } else { "" });
    }
    out.push_str("],\"files\":[");
    for (i, path) in state.files.iter().enumerate() {
        let text = path.to_str().ok_or_else(|| path.clone())?;
        if i > 0 {
            out.push(',');
        }
        write_json_string(&mut out, text);
    }
    let rebased_to = state.rebased_to.as_ref().map_or_else(|| "null".to_string(), |id| format!("\"{id}\""));
    let _ = writeln!(out, "],\"files_written\":{},\"rebased_to\":{rebased_to}}}", state.files_written);
    Ok(out)
}
