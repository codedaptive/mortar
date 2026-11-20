//! Git::worktree_add: add a worktree with an empty folder. It only runs git, so it has no unit tests;
//! tests/typed/worktrees.rs covers it: worktrees on a new branch and detached, each an empty folder holding only
//! .git and listed by worktree_list.

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
