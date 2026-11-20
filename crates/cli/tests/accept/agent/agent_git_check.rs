//! Step 4's check 3: no git write from outside Mortar. Real files: run_agent.rs runs it on every drive with
//! ACCEPT_AGENT set.

use std::fs;

use super::{non_read_git_calls, AgentFiles, AGENT_CHECKS};
use crate::report::Checks;

/// git.log holds at least one call (so the shim was on the agent's PATH) and non_read_git_calls finds none; the
/// saw text counts the calls and lists every one that is not a read.
pub fn agent_git_check(checks: &mut Checks, files: &AgentFiles) {
    let text = match fs::read_to_string(&files.git_log) {
        Ok(text) => text,
        Err(error) => {
            checks.check(AGENT_CHECKS[3], false, &format!("git.log unreadable: {error}"));
            return;
        }
    };
    let calls = text.lines().filter(|line| !line.trim().is_empty()).count();
    let outside = non_read_git_calls(&text);
    let saw = format!("{calls} git calls in {}; not reads: [{}]", files.git_log.display(), outside.join("; "));
    checks.check(AGENT_CHECKS[3], calls > 0 && outside.is_empty(), &saw);
}
