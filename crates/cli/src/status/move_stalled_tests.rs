//! move_stalled: a known index tree that differs from the new tip's tree is a stall; the same tree, or no tree
//! (an unmerged index), is not (S-97).

use gitio::Oid;

use super::move_stalled;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

const NEW_TREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OLD_TREE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn an_index_at_another_tree_is_stalled() {
    assert!(move_stalled(Some(&oid(OLD_TREE)), &oid(NEW_TREE)));
}

#[test]
fn an_index_at_the_new_tips_tree_is_not_stalled() {
    assert!(!move_stalled(Some(&oid(NEW_TREE)), &oid(NEW_TREE)));
}

#[test]
fn an_unmerged_index_is_not_stalled() {
    assert!(!move_stalled(None, &oid(NEW_TREE)));
}
