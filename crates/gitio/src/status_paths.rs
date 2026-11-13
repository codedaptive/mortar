//! StatusPaths: the paths of a folder that differ from its HEAD, by kind. A declaration only; read_status_paths_tests
//! exercises it.

use std::path::PathBuf;

/// A folder's changed, untracked and conflicted paths, each list in git's order. Ignored files are in none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusPaths {
    /// Paths changed in the index or the files (added, modified, deleted); a rename shows as its new and its old
    /// path.
    pub changed: Vec<PathBuf>,
    /// Files git does not track, each named (never only their folder).
    pub untracked: Vec<PathBuf>,
    /// Paths left unmerged in the index.
    pub conflicted: Vec<PathBuf>,
}
