//! version_text: The `--version` output: one line, no `next:`.

use oplog::write_json_string;

/// Plain "mortar <version> <commit>" and a newline, or the JSON object and a newline.
pub fn version_text(version: &str, commit: &str, json: bool) -> String {
    if !json {
        return format!("mortar {version} {commit}\n");
    }
    let mut out = String::from("{\"verb\":\"version\",\"ok\":true,\"version\":");
    write_json_string(&mut out, version);
    out.push_str(",\"commit\":");
    write_json_string(&mut out, commit);
    out.push_str("}\n");
    out
}
