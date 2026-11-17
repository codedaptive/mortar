//! Step 4's check 4: the agent used only the six verbs. Real files: run_agent.rs runs it on every drive with
//! ACCEPT_AGENT set.

use std::fs;
use std::path::Path;

use super::{agent_verbs_outside_six, mr_verb, parse_log_line, AgentFiles, AGENT_CHECKS};
use crate::report::Checks;

/// mr.log holds at least one call of the agent's and agent_verbs_outside_six finds none. The calls are read from
/// mr.log rather than the transcript: the wrappers log every call, while an agent tool's output need not show its
/// commands. The saw text lists the agent's verbs in order, every call outside the six, and the transcript's path.
pub fn agent_verbs_check(checks: &mut Checks, files: &AgentFiles, transcript: &Path) {
    let text = match fs::read_to_string(&files.mr_log) {
        Ok(text) => text,
        Err(error) => {
            checks.check(AGENT_CHECKS[4], false, &format!("mr.log unreadable: {error}"));
            return;
        }
    };
    let verbs: Vec<String> = text
        .lines()
        .filter_map(parse_log_line)
        .filter(|call| call.from_agent())
        .map(|call| mr_verb(&call.args).unwrap_or("(none)").to_string())
        .collect();
    let outside = agent_verbs_outside_six(&text);
    let saw = format!(
        "the agent's verbs [{}]; outside the six: [{}]; the agent's transcript: {}",
        verbs.join(", "),
        outside.join("; "),
        transcript.display()
    );
    checks.check(AGENT_CHECKS[4], !verbs.is_empty() && outside.is_empty(), &saw);
}
