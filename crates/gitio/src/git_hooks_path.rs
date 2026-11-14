//! Git::hooks_path: the folder git runs this repository's hooks from. It only runs git, so it has no unit tests;
//! tests/land_ops/local_reads.rs covers it ("hooks_path default, absolute and relative core.hooksPath, from a linked
//! worktree and a bare repo").

use crate::hooks_path_request::hooks_path_request;
use crate::path_line::path_line;
use crate::{Git, GitError};
use std::path::PathBuf;

impl Git {
    /// The absolute hooks folder for `repo`: the common dir's `hooks` unless core.hooksPath sets another; a relative
    /// core.hooksPath is resolved against the top of the worktree `repo` is in (a bare repository: its git dir).
    pub fn hooks_path(&self) -> Result<PathBuf, GitError> {
        let (args, opts) = hooks_path_request();
        path_line(&self.run(&args, &opts)?.stdout)
    }
}
