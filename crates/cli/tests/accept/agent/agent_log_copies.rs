//! Where step 4's wrapper logs are copied. Pure: copy_agent_logs.rs passes it the agent's files and the transcript.

use std::path::{Path, PathBuf};

use super::AgentFiles;

/// The git log, then the mr log: each its name, where it is (files.git_log, files.mr_log) and where its copy goes,
/// beside the transcript as `agent-git.log` and `agent-mr.log`, so the copies outlive the world.
pub fn agent_log_copies(files: &AgentFiles, transcript: &Path) -> [(&'static str, PathBuf, PathBuf); 2] {
    [
        ("git", files.git_log.clone(), transcript.with_file_name("agent-git.log")),
        ("mr", files.mr_log.clone(), transcript.with_file_name("agent-mr.log")),
    ]
}
