//! Git::set_index: set a folder's own index to a tree, leaving its files and HEAD untouched, and keep a sparse
//! folder's skip-worktree bits. It only runs git, so it has no unit tests of its own; the zone integration tests in
//! crates/gitio/tests/typed/set_index.rs cover it.

use crate::read_sparse_flag::read_sparse_flag;
use crate::set_index_request::set_index_request;
use crate::sparse_reapply_requests::sparse_reapply_requests;
use crate::{Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// Replaces the index of `folder`, a worktree of this repository, with `tree`; files and HEAD are not touched, and
    /// the same tree again changes nothing. The folder's sparse setting is read first. After the read-tree, a sparse
    /// folder (core.sparseCheckout true) gets `sparse-checkout reapply`, which marks every tracked path outside the
    /// sparse patterns skip-worktree without writing any file; an out-of-cone tracked file present on disk is left
    /// with no bit and its bytes. A folder with no core.sparseCheckout runs read-tree only; a folder with the setting
    /// on but no patterns runs reapply too, which changes no bit or file. A failing run is the runner's GitError
    /// carrying git's exit and stderr: the read-tree on an unknown object or one that is not a tree, the config read
    /// on an exit other than 0 or 1, the reapply on any non-zero exit; a config answer that is not a bool is a refusal
    /// with no exit. If the reapply fails, the index is already the tree, with the out-of-cone bits as read-tree left
    /// them (dropped), files and HEAD untouched, and calling set_index again finishes it. All the runs take this Git
    /// value's env after gitio removes GIT_INDEX_FILE, so a GIT_INDEX_FILE set there reaches every run: read-tree and
    /// reapply set that index instead of the folder's own, and reapply, right after the read-tree with no stat info,
    /// writes no working file. Git::commit_hooks relies on it in a worktree; tests/typed/commit_hooks.rs covers it.
    pub fn set_index(&self, folder: &Path, tree: &Oid) -> Result<(), GitError> {
        let [flag, reapply] = sparse_reapply_requests(folder)?;
        let (args, opts) = set_index_request(folder, tree)?;
        let sparse = read_sparse_flag(&self.run(&flag.0, &flag.1)?)?;
        self.run(&args, &opts)?;
        if sparse {
            self.run(&reapply.0, &reapply.1)?;
        }
        Ok(())
    }
}
