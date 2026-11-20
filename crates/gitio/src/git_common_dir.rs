//! Git::common_dir: the repository's shared git directory. It only runs git, so it has no unit tests;
//! tests/typed/worktrees.rs covers it: the main repository's .git from inside a linked worktree and from the main one.

use crate::path_line::path_line;
use crate::{Git, GitError, RunOpts};
use std::path::PathBuf;

impl Git {
    /// The absolute path of the git directory every worktree of the repository shares. `--path-format=absolute`
    /// because git prints a relative `.git` without it.
    pub fn common_dir(&self) -> Result<PathBuf, GitError> {
        let output = self.run(
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
            &RunOpts {
                read_only: true,
                ..RunOpts::default()
            },
        )?;
        path_line(&output.stdout)
    }
}
