//! Git::toplevel: the top folder of the worktree. It only runs git, so it has no unit tests; tests/typed/worktrees.rs
//! `worktree_lifecycle_sparse_paths_and_discovery_share_the_real_common_store` covers it: a linked worktree's top
//! from a subfolder, and the main worktree's top.

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
