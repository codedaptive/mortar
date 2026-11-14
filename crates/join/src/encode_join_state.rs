//! encode_join_state: a JoinState as the text of mortar-join.json. encode_join_state_tests covers the
//! layout, empty arrays, string escapes and the refusal of a non-UTF-8 path.
//! Its string writer is a copy of oplog's private write_json_string, a stopgap until oplog exports it (WO-oplog-7);
//! WO-join-2b switches to oplog's and deletes the copy (S-44).

use crate::join_state::JoinState;
use std::fmt::Write;
use std::path::PathBuf;

/// One JSON object and a trailing newline, keys in the order request_ref, request_tip, onto, replayed_tip,
/// commit, tree, remaining, files; ids as hex strings, remaining and files as arrays. A conflicted path that
/// is not UTF-8 cannot be written without loss, so the first such path is returned as the error.
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
    out.push_str("]}\n");
    Ok(out)
}

/// Appends `s` in double quotes, escaping '"', '\' and every char below U+0020 (\n \r \t \b \f in their
/// short forms, the rest as \u00XX); every other char is written as is.
/// A copy of oplog's write_json_string (crates/oplog/src/store/write_json_string.rs), which oplog does not export.
fn write_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            // Writing to a String cannot fail.
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
