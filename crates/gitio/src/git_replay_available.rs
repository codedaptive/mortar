//! Git::replay_available: whether this git has a replay that `replay` can use. It only runs git with a fixed argv,
//! so it has no unit tests; read_replay_probe_tests cover the reading, and Nagatha's integration test
//! "replay_available answering true on the installed git" is to cover the run.

use crate::read_replay_probe::read_replay_probe;
use crate::{Git, GitError, RunOpts};

impl Git {
    /// True when `git replay` exists and can print its ref moves (`--ref-action`). Exit 129 is an answer: older gits
    /// print the usage with it.
    pub fn replay_available(&self) -> Result<bool, GitError> {
        read_replay_probe(self.run(
            &["replay", "-h"],
            &RunOpts {
                read_only: true,
                answer_codes: vec![129],
                ..RunOpts::default()
            },
        ))
    }
}
