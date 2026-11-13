//! Git::toplevel: the top folder of the worktree. It only runs git, so it has no unit tests; Nagatha's integration
//! test "toplevel answering the worktree's top from a subfolder" is to cover it.

use crate::path_line::path_line;
use crate::{Git, GitError, RunOpts};
use std::path::PathBuf;

impl Git {
    /// The absolute path of the top of the worktree `repo` is in.
    pub fn toplevel(&self) -> Result<PathBuf, GitError> {
        let output = self.run(
            &["rev-parse", "--show-toplevel"],
            &RunOpts {
                read_only: true,
                ..RunOpts::default()
            },
        )?;
        path_line(&output.stdout)
    }
}
