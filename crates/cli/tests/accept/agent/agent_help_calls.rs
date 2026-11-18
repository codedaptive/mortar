//! The agent's help and version calls in mr.log, which check 4's report lists (04-agent.md, S-181).

use super::{help_or_version_call, parse_log_line};

/// From mr.log's text: each of the agent's calls (LogCall::from_agent) that is a help_or_version_call, as
/// LogCall::shown, in log order. Blank lines and lines that cannot be read are skipped; agent_verbs_outside_six
/// lists the unreadable ones.
pub fn agent_help_calls(log: &str) -> Vec<String> {
    log.lines()
        .filter_map(parse_log_line)
        .filter(|call| call.from_agent() && help_or_version_call(&call.args))
        .map(|call| call.shown())
        .collect()
}
