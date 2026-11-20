//! count_unsaved: changed and conflicted paths count as tracked, untracked files as untracked.

use std::path::PathBuf;

use gitio::StatusPaths;

use super::count_unsaved::count_unsaved;
use super::UnsavedCounts;

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

#[test]
fn empty_status_counts_zero() {
    let status = StatusPaths { changed: vec![], untracked: vec![], conflicted: vec![] };
    assert_eq!(count_unsaved(&status), UnsavedCounts { tracked: 0, untracked: 0 });
}

#[test]
fn changed_and_conflicted_count_as_tracked_untracked_apart() {
    let status = StatusPaths {
        changed: paths(&["a", "b"]),
        untracked: paths(&["x", "y/z", "w"]),
        conflicted: paths(&["c"]),
    };
    assert_eq!(count_unsaved(&status), UnsavedCounts { tracked: 3, untracked: 3 });
}

#[test]
fn conflicted_only_counts_as_tracked() {
    let status = StatusPaths { changed: vec![], untracked: vec![], conflicted: paths(&["c1", "c2"]) };
    assert_eq!(count_unsaved(&status), UnsavedCounts { tracked: 2, untracked: 0 });
}

#[test]
fn untracked_only_counts_as_untracked() {
    let status = StatusPaths { changed: vec![], untracked: paths(&["n1", "n2", "n3", "n4"]), conflicted: vec![] };
    assert_eq!(count_unsaved(&status), UnsavedCounts { tracked: 0, untracked: 4 });
}
