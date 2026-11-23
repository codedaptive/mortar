//! encode_join_state: a JoinState as the text of mortar-join.json. encode_join_state_tests covers the
//! layout, empty arrays, both flag values, a rebased tip and folder tree, a base with and without partners, a base
//! before with and without a base, string escapes and the refusal of a non-UTF-8 path; decode_join_state_tests covers
//! reading a hand-written text and a round trip through this encoder.

use crate::join_state::JoinState;
use crate::request_base::RequestBase;
use crate::write_partners::write_partners;
use gitio::Oid;
use oplog::write_json_string;
use std::fmt::Write;
use std::path::PathBuf;

/// One JSON object with no whitespace and a trailing newline, its 12 keys in the order request_ref, request_tip,
/// onto, replayed_tip, commit, tree, remaining, files, all_files, files_written, rebased_to, folder_tree; ids as hex
/// strings, remaining, files and all_files as arrays, files_written as `true` or `false`, rebased_to and folder_tree
/// as `null` or a hex string. A state with a base adds two keys after folder_tree: `base`, a hex string, and
/// `partners`, an array of `{"zone":<string>,"tip":<hex string>}` objects in the base's order. A state with a base
/// before then adds `base_before` and `partners_before` in the same forms. A state with neither writes the 12 keys
/// alone, as join's state does. A conflicted path that is not UTF-8 cannot be written
/// without loss, so the first such path (files first, then all_files) is returned as the error.
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
    out.push_str("],\"files\":");
    write_paths(&mut out, &state.files)?;
    out.push_str(",\"all_files\":");
    write_paths(&mut out, &state.all_files)?;
    let _ = write!(
        out,
        ",\"files_written\":{},\"rebased_to\":{},\"folder_tree\":{}",
        state.files_written,
        optional_id(state.rebased_to.as_ref()),
        optional_id(state.folder_tree.as_ref())
    );
    if let Some(base) = &state.base {
        write_base(&mut out, ("base", "partners"), base);
    }
    if let Some(before) = &state.base_before {
        write_base(&mut out, ("base_before", "partners_before"), before);
    }
    out.push_str("}\n");
    Ok(out)
}

/// `,"<base key>":"<hex>","<partners key>":[...]`, each partner as `{"zone":<string>,"tip":"<hex>"}`.
fn write_base(out: &mut String, (base_key, partners_key): (&str, &str), base: &RequestBase) {
    let _ = write!(out, ",\"{base_key}\":\"{}\",\"{partners_key}\":", base.base);
    write_partners(out, &base.partners);
}

/// `paths` as a JSON array of strings, or the first path that is not UTF-8.
fn write_paths(out: &mut String, paths: &[PathBuf]) -> Result<(), PathBuf> {
    out.push('[');
    for (i, path) in paths.iter().enumerate() {
        let text = path.to_str().ok_or_else(|| path.clone())?;
        if i > 0 {
            out.push(',');
        }
        write_json_string(out, text);
    }
    out.push(']');
    Ok(())
}

fn optional_id(id: Option<&Oid>) -> String {
    id.map_or_else(|| "null".to_string(), |id| format!("\"{id}\""))
}
