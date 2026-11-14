//! Git::set_index: set a folder's own index to a tree, leaving its files and HEAD untouched. It only runs git, so it
//! has no unit tests; Nagatha's integration test "set_index on a real repo setting the index to the tree, leaving a
//! dirty file's contents and HEAD unchanged, a second call a no-op, a bad tree (unknown object, not a tree) the typed
//! GitError with git's exit and stderr, and in a sparse folder skip-worktree bits as git gives them" is to cover it.

use crate::set_index_request::set_index_request;
use crate::{Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// Replaces the index of `folder`, a worktree of this repository, with `tree`; files and HEAD are not touched, and
    /// the same tree again changes nothing. The output is not parsed: read-tree writes nothing on success, and any
    /// non-zero exit (an unknown object, or one that is not a tree) is the runner's GitError carrying git's exit and
    /// stderr. The run takes this Git value's env after gitio removes GIT_INDEX_FILE, so a GIT_INDEX_FILE set there
    /// points read-tree at that index instead of the folder's own.
    pub fn set_index(&self, folder: &Path, tree: &Oid) -> Result<(), GitError> {
        let (args, opts) = set_index_request(folder, tree)?;
        self.run(&args, &opts).map(drop)
    }
}
