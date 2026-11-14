//! branch_move_for: at the new tip is AlreadyMoved, at the old tip is Move, anywhere else or no commit is
//! Elsewhere; equal tips are AlreadyMoved.

use gitio::Oid;

use super::branch_move_for::branch_move_for;
use super::BranchMove;

const OLD: &str = "0123456789abcdef0123456789abcdef01234567";
const NEW: &str = "fedcba9876543210fedcba9876543210fedcba98";
const OTHER: &str = "1111111111111111111111111111111111111111";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn at_the_new_tip_is_already_moved() {
    assert_eq!(branch_move_for(Some(&oid(NEW)), &oid(OLD), &oid(NEW)), BranchMove::AlreadyMoved);
}

#[test]
fn at_the_old_tip_is_move() {
    assert_eq!(branch_move_for(Some(&oid(OLD)), &oid(OLD), &oid(NEW)), BranchMove::Move);
}

#[test]
fn at_another_commit_is_elsewhere() {
    assert_eq!(branch_move_for(Some(&oid(OTHER)), &oid(OLD), &oid(NEW)), BranchMove::Elsewhere);
}

#[test]
fn no_commit_is_elsewhere() {
    assert_eq!(branch_move_for(None, &oid(OLD), &oid(NEW)), BranchMove::Elsewhere);
}

#[test]
fn equal_tips_are_already_moved() {
    assert_eq!(branch_move_for(Some(&oid(OLD)), &oid(OLD), &oid(OLD)), BranchMove::AlreadyMoved);
}
