//! An index holding a merge's conflicts: Unmerged names each conflicted path once, from the top, in git's order.
use crate::fixture::conflicted;
use gitio::IndexTreeError;
use std::path::PathBuf;

/// The fixture's conflicted paths in git's (byte) order; its cleanly merged clean.txt and dir/clean.txt are not here.
fn conflicts() -> Vec<PathBuf> {
    ["a.txt", "dir/sp ace.txt", "z.txt"].map(PathBuf::from).to_vec()
}

#[test]
fn a_merge_conflict_gives_unmerged_with_exactly_the_conflicted_paths() {
    let r = conflicted();
    assert_eq!(r.git.index_tree(&r.path), Err(IndexTreeError::Unmerged { paths: conflicts() }));
}

#[test]
fn a_subfolder_of_a_conflicted_worktree_gives_every_conflicted_path_from_the_top() {
    let r = conflicted();
    assert_eq!(r.git.index_tree(&r.path.join("dir")), Err(IndexTreeError::Unmerged { paths: conflicts() }));
}
