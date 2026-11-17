//! integration_move_stopped: a stall only when integration moved past old and the folder's index is still at old's
//! tree; each input flips the answer.

use gitio::Oid;

use super::integration_move_stopped;

const OLD: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";
const OLD_TREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TIP_TREE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const STAGED_TREE: &str = "cccccccccccccccccccccccccccccccccccccccc";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn an_index_at_olds_tree_with_integration_past_old_is_a_stall() {
    assert!(integration_move_stopped(Some(&oid(OLD_TREE)), &oid(OLD), &oid(OLD_TREE), &oid(TIP)));
}

#[test]
fn integration_still_at_old_is_no_stall() {
    assert!(!integration_move_stopped(Some(&oid(OLD_TREE)), &oid(OLD), &oid(OLD_TREE), &oid(OLD)));
}

#[test]
fn an_unmerged_index_is_never_a_stall() {
    assert!(!integration_move_stopped(None, &oid(OLD), &oid(OLD_TREE), &oid(TIP)));
}

#[test]
fn an_index_at_the_tips_tree_is_no_stall() {
    assert!(!integration_move_stopped(Some(&oid(TIP_TREE)), &oid(OLD), &oid(OLD_TREE), &oid(TIP)));
}

#[test]
fn an_index_with_staged_edits_is_no_stall() {
    assert!(!integration_move_stopped(Some(&oid(STAGED_TREE)), &oid(OLD), &oid(OLD_TREE), &oid(TIP)));
}

#[test]
fn old_tree_is_what_the_index_is_compared_with() {
    assert!(integration_move_stopped(Some(&oid(STAGED_TREE)), &oid(OLD), &oid(STAGED_TREE), &oid(TIP)));
    assert!(!integration_move_stopped(Some(&oid(OLD_TREE)), &oid(OLD), &oid(STAGED_TREE), &oid(TIP)));
}

#[test]
fn old_is_what_the_tip_is_compared_with() {
    assert!(!integration_move_stopped(Some(&oid(OLD_TREE)), &oid(TIP), &oid(OLD_TREE), &oid(TIP)));
    assert!(integration_move_stopped(Some(&oid(OLD_TREE)), &oid(TIP), &oid(OLD_TREE), &oid(OLD)));
}
