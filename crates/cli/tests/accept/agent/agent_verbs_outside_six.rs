//! The agent's `mr`/`mortar` calls check 4 fails on (04-agent.md).

use super::{help_or_version_call, mr_verb, parse_log_line};

/// The six verbs an agent uses (MISSION line 11).
const SIX: [&str; 6] = ["status", "save", "join", "resolve", "need", "undo"];

/// From mr.log's text: each of the agent's calls (LogCall::from_agent) that names a verb (mr_verb) not among the
/// six, as LogCall::shown, in log order; each non-blank line that cannot be read as "unreadable mr.log line:
/// <line>". A help or version call (help_or_version_call) reads no repository and changes nothing, so it is not
/// outside the six (S-181); a call that names no verb (a bare `mr`) uses none, so it is not listed either (S-172).
pub fn agent_verbs_outside_six(log: &str) -> Vec<String> {
    log.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| match parse_log_line(line) {
            None => Some(format!("unreadable mr.log line: {line}")),
            Some(call) if !call.from_agent() || help_or_version_call(&call.args) => None,
            Some(call) => match mr_verb(&call.args) {
                Some(verb) if !SIX.contains(&verb) => Some(call.shown()),
                _ => None,
            },
        })
        .collect()
}
