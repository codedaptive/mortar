//! Git::worktree_fill: write every file of a worktree's commit into its folder. It only runs git, so it has no unit
//! tests; tests/land_ops/worktree_fill.rs covers it ("worktree_fill writing every file with shared
//! core.sparseCheckout=true, status empty").

use crate::worktree_fill_request::worktree_fill_request;
use crate::{Git, GitError};
use std::path::Path;

impl Git {
    /// Fills `folder` (a worktree of this repository, such as one worktree_add left empty) with every file of its
    /// HEAD commit, whatever sparse settings apply, so `status` there shows nothing. Fill only an empty worktree from
    /// worktree_add: local edits to tracked files are overwritten with the commit's, not refused. A missing folder is
    /// git's own error (exit 128).
    pub fn worktree_fill(&self, folder: &Path) -> Result<(), GitError> {
        let (args, opts) = worktree_fill_request(folder)?;
        self.run(&args, &opts).map(drop)
    }
}
