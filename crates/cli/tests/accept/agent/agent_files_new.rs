//! Names step 4's wrapper folders and logs. Pure: run_agent.rs passes it the world's root.

use std::path::Path;

use super::AgentFiles;

impl AgentFiles {
    /// Everything in `<root>/agent-trace`: the folders `bin` and `shim`, and the files `mr.log`, `git.log`,
    /// `conflict-done` and `drive-out.txt`.
    pub fn new(root: &Path) -> AgentFiles {
        let trace = root.join("agent-trace");
        AgentFiles {
            bin: trace.join("bin"),
            shim: trace.join("shim"),
            mr_log: trace.join("mr.log"),
            git_log: trace.join("git.log"),
            once: trace.join("conflict-done"),
            drive_out: trace.join("drive-out.txt"),
        }
    }
}
