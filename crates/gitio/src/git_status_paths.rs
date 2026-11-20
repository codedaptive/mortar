//! Git::status_paths: a folder's changed, untracked and conflicted paths. It only runs git, so it has no unit tests;
//! snapshot's tests/snapshot/formatter_paths.rs
//! (formatter_gets_only_regular_saved_nonconflicted_changes_not_deleted_refused_excluded_links_or_submodules) covers it
//! through the formatter: it is handed the changed and untracked files, not the conflicted one.

use crate::read_status_paths::read_status_paths;
use crate::status_paths_request::status_paths_request;
use crate::{Git, GitError, StatusPaths};
use std::path::Path;

impl Git {
    /// The paths of `folder`, a worktree of this repository, that differ from its HEAD: changed (in the index or the
    /// files; a rename as its old and new path), untracked (each file named) and conflicted, each in git's order.
    /// Ignored files are left out; a clean folder gives three empty lists.
    pub fn status_paths(&self, folder: &Path) -> Result<StatusPaths, GitError> {
        let (args, opts) = status_paths_request(folder)?;
        read_status_paths(&self.run(&args, &opts)?.stdout)
    }
}
