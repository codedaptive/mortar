//! Whether the conflict the drive sets up reached the agent, read from mr.log's order.

use super::{mr_verb, parse_log_line, LogCall};

/// From mr.log's text (lines that cannot be read are left out). The `mr` wrapper runs the drive's change on the
/// agent's first `mr join` and logs that join after it, so Ok when the drive's `mr join` (parent "drive") is logged
/// before the agent's first `mr join` (LogCall::from_agent) and the agent ran `mr resolve`. Otherwise an error
/// saying which of those is missing, the first missing one in that order.
pub fn conflict_seen(log: &str) -> Result<(), String> {
    let calls: Vec<LogCall> = log.lines().filter_map(parse_log_line).collect();
    let drive_join = calls.iter().position(|call| call.parent == "drive" && mr_verb(&call.args) == Some("join"));
    let agent = |verb: &str| calls.iter().position(|call| call.from_agent() && mr_verb(&call.args) == Some(verb));
    match (drive_join, agent("join"), agent("resolve")) {
        (None, _, _) => Err("the drive's mr join in rc never ran".to_string()),
        (_, None, _) => Err("the agent ran no mr join".to_string()),
        (Some(drive), Some(first), _) if first < drive => {
            Err("the agent's first mr join was logged before the drive's, so it met no conflict".to_string())
        }
        (_, _, None) => Err("the agent ran no mr resolve".to_string()),
        _ => Ok(()),
    }
}
