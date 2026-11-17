//! The git calls check 3 fails on (04-agent.md).

use super::{git_call_is_read, parse_log_line};

/// From git.log's text: each call whose parent is not a Mortar process (base name `mr` or `mortar`) and that is
/// not a read (git_call_is_read), as LogCall::shown, in log order; each non-blank line that cannot be read as
/// "unreadable git.log line: <line>". Empty when every call outside Mortar reads.
pub fn non_read_git_calls(log: &str) -> Vec<String> {
    log.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| match parse_log_line(line) {
            None => Some(format!("unreadable git.log line: {line}")),
            Some(call) if call.parent == "mr" || call.parent == "mortar" => None,
            Some(call) if git_call_is_read(&call.args) => None,
            Some(call) => Some(call.shown()),
        })
        .collect()
}
