//! A folder's status paths, counted as unsaved work.

use gitio::StatusPaths;

use super::UnsavedCounts;

/// tracked is the changed paths plus the conflicted ones: a conflicted path is tracked work not yet saved. untracked
/// is the untracked files. A rename counts as two paths, its old and its new, since gitio's status runs
/// `--no-renames`.
pub(crate) fn count_unsaved(paths: &StatusPaths) -> UnsavedCounts {
    UnsavedCounts {
        tracked: (paths.changed.len() + paths.conflicted.len()) as u64,
        untracked: paths.untracked.len() as u64,
    }
}
