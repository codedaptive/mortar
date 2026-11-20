//! guard_config_text: guard.json's text for a set of branches.
//! No sibling unit test: every test of it runs oplog's write_json_string. tests/land_rit/guard_codec.rs checks
//! its exact bytes through guard_install and round-trips many names through read_guard_config.
//! tests/land_rit/guard_installation.rs checks that a repeat install leaves guard.json as it was.

use oplog::write_json_string;

use crate::LandRefs;

/// One line, `{"origin_branch":..,"integration_branch":..,"remote":..}` and a newline, each string written by
/// oplog's write_json_string. decide_config_write compares guard.json's bytes with this text, so it is the one form
/// written.
pub(crate) fn guard_config_text(refs: &LandRefs) -> String {
    let mut out = String::from("{\"origin_branch\":");
    write_json_string(&mut out, &refs.origin_branch);
    out.push_str(",\"integration_branch\":");
    write_json_string(&mut out, &refs.integration_branch);
    out.push_str(",\"remote\":");
    write_json_string(&mut out, &refs.remote);
    out.push_str("}\n");
    out
}
