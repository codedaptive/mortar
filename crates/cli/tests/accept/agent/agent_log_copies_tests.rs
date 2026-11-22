use std::path::{Path, PathBuf};

use super::{agent_log_copies, AgentFiles};

#[test]
fn git_then_mr_from_the_trace_folder_to_beside_the_transcript() {
    let files = AgentFiles::new(Path::new("/w"));
    let expected = [
        ("git", PathBuf::from("/w/agent-trace/git.log"), PathBuf::from("/r/agent-git.log")),
        ("mr", PathBuf::from("/w/agent-trace/mr.log"), PathBuf::from("/r/agent-mr.log")),
    ];
    assert_eq!(agent_log_copies(&files, Path::new("/r/agent-transcript.txt")), expected);
}
