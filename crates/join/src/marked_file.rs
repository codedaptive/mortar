//! MarkedFile: a conflicted file and the conflict-marker lines left in it. A declaration only; decide_resolve_tests
//! and join_run_error_display_tests exercise it.

use std::path::PathBuf;

/// A conflicted path, relative to the folder, and the 1-based numbers of its lines that are conflict markers (empty
/// when none remain, or when the file was deleted as its resolution).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkedFile {
    pub path: PathBuf,
    pub lines: Vec<usize>,
}
