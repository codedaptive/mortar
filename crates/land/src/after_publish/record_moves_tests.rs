//! record_moves: none, either one alone, both on distinct refs (consumed first), and both on one ref (carried only).

use super::record_moves;
use gitio::Oid;
use oplog::RefMove;

const CONSUMED_REF: &str = "refs/mortar/replayed/land/1111111111111111111111111111111111111111";
const CARRIED_REF: &str = "refs/mortar/replayed/land/2222222222222222222222222222222222222222";
const RECORDED: &str = "3333333333333333333333333333333333333333";
const REPLAYED: &str = "4444444444444444444444444444444444444444";
const ZERO: &str = "0000000000000000000000000000000000000000";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn delete(refname: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(oid(RECORDED)), new: oid(ZERO) }
}

fn carry(refname: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(oid(RECORDED)), new: oid(REPLAYED) }
}

#[test]
fn neither_is_no_move() {
    assert_eq!(record_moves(None, None), vec![]);
}

#[test]
fn the_consumed_delete_alone_is_its_one_move() {
    assert_eq!(record_moves(Some(delete(CONSUMED_REF)), None), vec![delete(CONSUMED_REF)]);
}

#[test]
fn the_carried_move_alone_is_its_one_move() {
    assert_eq!(record_moves(None, Some(carry(CARRIED_REF))), vec![carry(CARRIED_REF)]);
}

#[test]
fn both_on_distinct_refs_are_the_consumed_delete_then_the_carried_move() {
    assert_eq!(
        record_moves(Some(delete(CONSUMED_REF)), Some(carry(CARRIED_REF))),
        vec![delete(CONSUMED_REF), carry(CARRIED_REF)]
    );
}

#[test]
fn both_on_one_ref_are_the_carried_move_only() {
    assert_eq!(record_moves(Some(delete(CARRIED_REF)), Some(carry(CARRIED_REF))), vec![carry(CARRIED_REF)]);
}
