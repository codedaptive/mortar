//! is_saved: a folder is saved only when its tree equals its branch tip's tree (S-144); a tree that equals only the
//! snapshot's tree is not saved.

use gitio::Oid;

use super::is_saved;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

const TIP_TREE: &str = "3333333333333333333333333333333333333333";
const SNAPSHOT_TREE: &str = "4444444444444444444444444444444444444444";
const OTHER_TREE: &str = "5555555555555555555555555555555555555555";

#[test]
fn a_folder_tree_equal_to_the_tip_tree_is_saved() {
    assert!(is_saved(&oid(TIP_TREE), &oid(TIP_TREE)));
}

/// After an undo of a save, or the harness's end-of-turn snapshot, the folder's tree is the snapshot's and not the
/// tip's: kept by a snapshot is not kept as a commit.
#[test]
fn a_folder_tree_equal_to_the_snapshot_tree_but_not_the_tip_tree_is_not_saved() {
    assert!(!is_saved(&oid(SNAPSHOT_TREE), &oid(TIP_TREE)));
}

#[test]
fn a_folder_tree_other_than_the_tip_tree_is_not_saved() {
    assert!(!is_saved(&oid(OTHER_TREE), &oid(TIP_TREE)));
    assert!(!is_saved(&oid(TIP_TREE), &oid(OTHER_TREE)));
}
