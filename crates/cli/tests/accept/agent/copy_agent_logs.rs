//! Copies step 4's wrapper logs beside the transcript. Real files with no sibling test: run_agent.rs calls it on every
//! drive with ACCEPT_AGENT set whose set-up made both request folders and wrote the wrappers, after the agent ran.

use std::path::Path;

use super::{agent_log_copies, AgentFiles};
use crate::report::Checks;

/// Copies each of agent_log_copies' logs to its place, replacing an earlier copy, and names it or says why it was
/// not copied on a log line: a copy that fails never fails the step.
pub fn copy_agent_logs(checks: &mut Checks, files: &AgentFiles, transcript: &Path) {
    for (name, from, to) in agent_log_copies(files, transcript) {
        match std::fs::copy(&from, &to) {
            Ok(_) => checks.log(&format!("the agent's {name} log: {}", to.display())),
            Err(error) => checks.log(&format!(
                "the agent's {name} log was not copied from {} to {}: {error}",
                from.display(),
                to.display()
            )),
        }
    }
}
