//! The config as one line of JSON.

use oplog::write_json_string;

use super::Config;
use crate::json::write_json_strings;

/// One JSON object on one line, then "\n": origin_branch, integration_branch, remote, zones (in name order,
/// each `{"paths":[...]}`), root_files, build_dirs, cap_bytes, floor_bytes, formatter (null, or
/// `{"command":[...],"timeout_seconds":n}`). decode_config reads it back equal.
pub fn encode_config(config: &Config) -> String {
    let mut out = String::from("{\"origin_branch\":");
    write_json_string(&mut out, &config.origin_branch);
    out.push_str(",\"integration_branch\":");
    write_json_string(&mut out, &config.integration_branch);
    out.push_str(",\"remote\":");
    write_json_string(&mut out, &config.remote);
    out.push_str(",\"zones\":{");
    for (i, (name, zone)) in config.zones.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_json_string(&mut out, name);
        out.push_str(":{\"paths\":");
        write_json_strings(&mut out, &zone.paths);
        out.push('}');
    }
    out.push_str("},\"root_files\":");
    write_json_strings(&mut out, &config.root_files);
    out.push_str(",\"build_dirs\":");
    write_json_strings(&mut out, &config.build_dirs);
    out.push_str(&format!(",\"cap_bytes\":{},\"floor_bytes\":{},\"formatter\":", config.cap_bytes, config.floor_bytes));
    match &config.formatter {
        Some(formatter) => {
            out.push_str("{\"command\":");
            write_json_strings(&mut out, &formatter.command);
            out.push_str(&format!(",\"timeout_seconds\":{}}}", formatter.timeout_seconds));
        }
        None => out.push_str("null"),
    }
    out.push_str("}\n");
    out
}
