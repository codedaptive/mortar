//! move_stalled: an index still at the old tip's tree, or at the op's recorded folder_tree, is a stall; an index at
//! the new tip's tree, at any other tree (staged edits), or with no tree (an unmerged index) is not.

use gitio::Oid;

use super::move_stalled;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

const NEW_TREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OLD_TREE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const FOLDER_TREE: &str = "cccccccccccccccccccccccccccccccccccccccc";
const STAGED_TREE: &str = "dddddddddddddddddddddddddddddddddddddddd";

#[test]
fn an_index_at_the_old_tips_tree_is_stalled() {
    assert!(move_stalled(Some(&oid(OLD_TREE)), &oid(OLD_TREE), None, &oid(NEW_TREE)));
}

#[test]
fn an_index_at_the_recorded_folder_tree_is_stalled() {
    assert!(move_stalled(Some(&oid(FOLDER_TREE)), &oid(OLD_TREE), Some(&oid(FOLDER_TREE)), &oid(NEW_TREE)));
}

#[test]
fn an_index_at_the_new_tips_tree_is_not_stalled() {
    assert!(!move_stalled(Some(&oid(NEW_TREE)), &oid(OLD_TREE), Some(&oid(FOLDER_TREE)), &oid(NEW_TREE)));
}

#[test]
fn an_edit_staged_after_a_finished_move_is_not_stalled() {
    assert!(!move_stalled(Some(&oid(STAGED_TREE)), &oid(OLD_TREE), None, &oid(NEW_TREE)));
}

#[test]
fn an_edit_staged_after_a_finished_resolve_move_is_not_stalled() {
    assert!(!move_stalled(Some(&oid(STAGED_TREE)), &oid(OLD_TREE), Some(&oid(FOLDER_TREE)), &oid(NEW_TREE)));
}

#[test]
fn an_unmerged_index_is_not_stalled() {
    assert!(!move_stalled(None, &oid(OLD_TREE), Some(&oid(FOLDER_TREE)), &oid(NEW_TREE)));
}

#[test]
fn an_index_at_the_new_tree_is_not_stalled_when_old_and_new_trees_match() {
    assert!(!move_stalled(Some(&oid(NEW_TREE)), &oid(NEW_TREE), None, &oid(NEW_TREE)));
}
