//! build_output_json: each build dir's outcome as one JSON object, for the `build_output` member of a report.

use folders::BuildDirClone;
use oplog::write_json_string;

/// One encoded object per clone, in the order given, for `OutValue::Raw`: `{"dir":..,"outcome":..}` with outcome
/// "shared", "absent", "kept" or "refused"; a refused dir adds `"why":..` after the outcome.
pub fn build_output_json(clones: &[BuildDirClone]) -> Vec<String> {
    clones
        .iter()
        .map(|clone| {
            let (dir, outcome, why) = match clone {
                BuildDirClone::Shared { dir } => (dir, "shared", None),
                BuildDirClone::Absent { dir } => (dir, "absent", None),
                BuildDirClone::Kept { dir } => (dir, "kept", None),
                BuildDirClone::Refused { dir, why } => (dir, "refused", Some(why)),
            };
            let mut out = String::from("{\"dir\":");
            write_json_string(&mut out, dir);
            out.push_str(",\"outcome\":\"");
            out.push_str(outcome);
            out.push('"');
            if let Some(why) = why {
                out.push_str(",\"why\":");
                write_json_string(&mut out, why);
            }
            out.push('}');
            out
        })
        .collect()
}
