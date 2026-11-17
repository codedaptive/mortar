use std::path::{Path, PathBuf};

use super::AgentFiles;

#[test]
fn every_file_is_under_the_roots_agent_trace_folder() {
    let files = AgentFiles::new(Path::new("/w"));
    let expected = AgentFiles {
        bin: PathBuf::from("/w/agent-trace/bin"),
        shim: PathBuf::from("/w/agent-trace/shim"),
        mr_log: PathBuf::from("/w/agent-trace/mr.log"),
        git_log: PathBuf::from("/w/agent-trace/git.log"),
        once: PathBuf::from("/w/agent-trace/conflict-done"),
        drive_out: PathBuf::from("/w/agent-trace/drive-out.txt"),
    };
    assert_eq!(files, expected);
}
