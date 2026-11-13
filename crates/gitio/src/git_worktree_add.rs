//! Git::worktree_add: add a worktree with an empty folder. It only runs git, so it has no unit tests; Nagatha's
//! integration test "worktree_add on a new branch and detached, each listed by worktree_list with an empty folder"
//! is to cover it.

use crate::worktree_add_request::worktree_add_request;
use crate::{Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// Adds a worktree at `path` on `commit`, creating `branch` there when given, else detached. The folder is left
    /// empty (`--no-checkout`): set its sparse paths with `sparse_set`, which fills it.
    pub fn worktree_add(&self, path: &Path, commit: &Oid, branch: Option<&str>) -> Result<(), GitError> {
        let (args, opts) = worktree_add_request(path, commit, branch)?;
        self.run(&args, &opts).map(drop)
    }
}
