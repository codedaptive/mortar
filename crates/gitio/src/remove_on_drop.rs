//! RemoveOnDrop: a temporary index that is removed when it goes out of scope. It only touches the file system, so it
//! has no unit tests; Nagatha's integration tests of snapshot_tree and tree_with_paths, each checking that no temp
//! index is left behind, are to cover it.

use std::path::PathBuf;

/// Names a temporary index and removes that file when dropped, by any return path. A failed removal cannot be
/// reported from drop; a file that was never made needs nothing.
pub(crate) struct RemoveOnDrop(pub(crate) PathBuf);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
