//! The agent's `mr`/`mortar` calls check 4 fails on (04-agent.md).

use super::{mr_verb, parse_log_line};

/// The six verbs an agent uses (MISSION line 11).
const SIX: [&str; 6] = ["status", "save", "join", "resolve", "need", "undo"];

/// From mr.log's text: each of the agent's calls (LogCall::from_agent) that names a verb (mr_verb) not among the
/// six, as LogCall::shown, in log order; each non-blank line that cannot be read as "unreadable mr.log line:
/// <line>". A call that names no verb (`mr --help`) uses none, so it is not listed.
pub fn agent_verbs_outside_six(log: &str) -> Vec<String> {
    log.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| match parse_log_line(line) {
            None => Some(format!("unreadable mr.log line: {line}")),
            Some(call) if !call.from_agent() => None,
            Some(call) => match mr_verb(&call.args) {
                Some(verb) if !SIX.contains(&verb) => Some(call.shown()),
                _ => None,
            },
        })
        .collect()
}
