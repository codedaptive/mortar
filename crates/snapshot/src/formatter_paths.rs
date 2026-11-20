//! formatter_paths: the paths a snapshot hands its formatter.

use gitio::StatusPaths;
use std::collections::HashSet;
use std::path::PathBuf;

/// The changed paths, then the untracked ones, in status order, each at most once (the first place kept), keeping
/// only those in `saved` (paths the snapshot would save differently from HEAD, so excluded and refused paths are
/// gone), in `files` (paths that are regular files now, so a deleted path or a rename's old side is gone) and not
/// conflicted: a conflicted path is not formatted even when it is also listed as changed.
pub(crate) fn formatter_paths(
    status: &StatusPaths,
    saved: &HashSet<PathBuf>,
    files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    status
        .changed
        .iter()
        .chain(&status.untracked)
        .filter(|p| saved.contains(*p) && files.contains(*p) && !status.conflicted.contains(*p))
        .filter(|p| seen.insert(*p))
        .cloned()
        .collect()
}
