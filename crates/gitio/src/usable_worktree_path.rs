//! usable_worktree_path: refuse a worktree path git could not be given.

use crate::no_nul::no_nul;
use crate::GitError;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Ok for a non-empty path without a NUL. An empty path names no folder, and a NUL cannot pass through argv; each is
/// refused naming the path.
pub(crate) fn usable_worktree_path(path: &Path) -> Result<(), GitError> {
    if path.as_os_str().is_empty() {
        return Err(GitError::refusal(Vec::new(), format!("not a usable worktree path (empty): {path:?}")));
    }
    no_nul("worktree path", path.as_os_str().as_bytes())
}
