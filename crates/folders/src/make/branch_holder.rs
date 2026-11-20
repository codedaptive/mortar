//! Which worktree, if any, has a branch checked out.

use std::path::{Path, PathBuf};

use gitio::Worktree;

/// The path of the first worktree other than `path` whose branch is the full ref `branch_ref`
/// (`refs/heads/<name>`), or None. Paths are compared as given.
pub fn branch_holder(worktrees: &[Worktree], branch_ref: &str, path: &Path) -> Option<PathBuf> {
    worktrees
        .iter()
        .find(|w| w.branch.as_deref() == Some(branch_ref) && w.path != path)
        .map(|w| w.path.clone())
}
