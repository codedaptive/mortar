//! temp_index_path: where a temporary index (Git::snapshot_tree, Git::tree_with_paths, Git::index_tree) lives.

use std::path::{Path, PathBuf};

/// `<dir>/mortar-index-<pid>-<stamp>-<serial>`. The process id keeps processes apart, the serial keeps two temporary
/// indexes of one process apart, and the stamp keeps a reused process id from meeting a file a dead process left
/// behind.
pub(crate) fn temp_index_path(dir: &Path, pid: u32, stamp: u128, serial: u64) -> PathBuf {
    dir.join(format!("mortar-index-{pid}-{stamp}-{serial}"))
}
