//! Git::snapshot_tree: save a folder's files as a tree without touching its index. It only runs git, so it has no
//! unit tests; Nagatha's integration test "snapshot_tree in a sparse folder keeping out-of-cone files, applying
//! excludes, leaving the folder's index unchanged and no temp index behind" is to cover it.

use crate::oid_line::oid_line;
use crate::snapshot_tree_requests::snapshot_tree_requests;
use crate::temp_index_path::temp_index_path;
use crate::{Git, GitError, Oid};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Counts this process's snapshots, so two at once never share a temp index.
static SERIAL: AtomicU64 = AtomicU64::new(0);

impl Git {
    /// The tree of every file in the folder (`self.repo`, which must be the folder's top), starting from `base` so
    /// files outside a sparse cone are kept, minus the `excludes` pathspecs. Writes objects; moves no ref.
    pub fn snapshot_tree(&self, base: &Oid, excludes: &[String]) -> Result<Oid, GitError> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let index = RemoveOnDrop(temp_index_path(
            &std::env::temp_dir(),
            std::process::id(),
            stamp,
            SERIAL.fetch_add(1, Ordering::Relaxed),
        ));
        let [read_tree, add, write_tree] = snapshot_tree_requests(base, excludes, &index.0)?;
        self.run(&read_tree.0, &read_tree.1)?;
        self.run(&add.0, &add.1)?;
        oid_line(&self.run(&write_tree.0, &write_tree.1)?.stdout)
    }
}

/// Removes the temp index when the snapshot returns, by any path. A failed removal cannot be reported from drop;
/// a file that was never made (an early refusal) is the usual case and needs nothing.
struct RemoveOnDrop(PathBuf);

impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
