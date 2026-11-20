//! One line of the user's savings.jsonl.

use std::path::Path;

use oplog::write_json_string;

/// `{"time":…,"folder":…,"verb":…,"output_tokens":X,"baseline_tokens":B}` and a newline: these snake_case keys in
/// this order, the one shape the codeteam deck reads from every tool. The folder goes through to_string_lossy.
pub fn savings_log_line(time: &str, folder: &Path, verb: &str, output_tokens: u64, baseline_tokens: u64) -> String {
    let mut out = String::from("{\"time\":");
    write_json_string(&mut out, time);
    out.push_str(",\"folder\":");
    write_json_string(&mut out, &folder.to_string_lossy());
    out.push_str(",\"verb\":");
    write_json_string(&mut out, verb);
    out.push_str(&format!(",\"output_tokens\":{output_tokens},\"baseline_tokens\":{baseline_tokens}}}\n"));
    out
}
