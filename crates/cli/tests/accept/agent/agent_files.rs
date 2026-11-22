//! Where step 4 keeps the agent's wrappers and logs. Data only: AgentFiles::new (agent_files_new.rs) names them
//! under the world's root, and run_agent.rs uses them on every drive with ACCEPT_AGENT set whose set-up made both
//! request folders.

use std::path::PathBuf;

/// `bin` holds the `mr` and `mortar` wrappers, `shim` the logging `git`; `mr_log` and `git_log` are what they log,
/// and run_agent copies both beside the transcript (copy_agent_logs);
/// `once` marks that the drive's conflicting change ran; `drive_out` takes that change's output, which is no part
/// of the agent's transcript.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentFiles {
    pub bin: PathBuf,
    pub shim: PathBuf,
    pub mr_log: PathBuf,
    pub git_log: PathBuf,
    pub once: PathBuf,
    pub drive_out: PathBuf,
}
