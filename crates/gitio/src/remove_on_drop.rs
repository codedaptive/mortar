//! RemoveOnDrop: a temporary index that is removed when it goes out of scope. It only touches the file system, so it
//! has no unit tests; the zone's integration tests that check no temp index is left behind cover it: those of
//! snapshot_tree and tree_with_paths (tests/typed/snapshot.rs, tree_with_paths.rs) and of index_tree
//! (tests/index_tree/untouched.rs).

use std::path::PathBuf;

/// Names a temporary index and removes that file when dropped, by any return path. A failed removal cannot be
/// reported from drop; a file that was never made needs nothing.
pub(crate) struct RemoveOnDrop(pub(crate) PathBuf);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
