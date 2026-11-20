//! Git::worktree_remove: remove a worktree and its folder. It only runs git, so it has no unit tests;
//! tests/typed/worktrees.rs `worktree_lifecycle_sparse_paths_and_discovery_share_the_real_common_store` covers it:
//! a modified worktree refused without force with its edit kept, then removed with its folder by force.

use crate::worktree_remove_request::worktree_remove_request;
use crate::{Git, GitError};
use std::path::Path;

impl Git {
    /// Removes the worktree at `path` and deletes its folder. Without `force` git refuses a worktree with changes or
    /// untracked files, and a worktree added with `--no-checkout` (as `worktree_add` does) counts as changed until it
    /// is checked out; whether to force is the caller's choice.
    pub fn worktree_remove(&self, path: &Path, force: bool) -> Result<(), GitError> {
        let (args, opts) = worktree_remove_request(path, force)?;
        self.run(&args, &opts).map(drop)
    }
}
