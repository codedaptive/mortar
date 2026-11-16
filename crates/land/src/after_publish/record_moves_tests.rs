//! record_moves: none, deletes alone, writes alone, deletes then writes on distinct refs (each in its own order), and
//! a delete of a ref a write also moves left out (the write wins) while the other deletes stay.

use super::record_moves;
use gitio::Oid;
use oplog::RefMove;

const REF_1: &str = "refs/mortar/replayed/land/1111111111111111111111111111111111111111";
const REF_2: &str = "refs/mortar/replayed/land/2222222222222222222222222222222222222222";
const REF_3: &str = "refs/mortar/replayed/land/3333333333333333333333333333333333333333";
const REF_4: &str = "refs/mortar/replayed/land/4444444444444444444444444444444444444444";
const RECORDED: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const REPLAYED: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ZERO: &str = "0000000000000000000000000000000000000000";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn delete(refname: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(oid(RECORDED)), new: oid(ZERO) }
}

fn write(refname: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(oid(RECORDED)), new: oid(REPLAYED) }
}

#[test]
fn neither_is_no_move() {
    assert_eq!(record_moves(vec![], vec![]), vec![]);
}

#[test]
fn deletes_alone_are_the_moves_in_their_order() {
    assert_eq!(record_moves(vec![delete(REF_2), delete(REF_1)], vec![]), vec![delete(REF_2), delete(REF_1)]);
}

#[test]
fn writes_alone_are_the_moves_in_their_order() {
    assert_eq!(record_moves(vec![], vec![write(REF_2), write(REF_1)]), vec![write(REF_2), write(REF_1)]);
}

#[test]
fn deletes_and_writes_on_distinct_refs_are_the_deletes_then_the_writes() {
    assert_eq!(
        record_moves(vec![delete(REF_3), delete(REF_1)], vec![write(REF_4), write(REF_2)]),
        vec![delete(REF_3), delete(REF_1), write(REF_4), write(REF_2)]
    );
}

#[test]
fn a_delete_of_a_ref_a_write_moves_is_left_out_and_the_other_deletes_stay() {
    assert_eq!(
        record_moves(vec![delete(REF_1), delete(REF_2), delete(REF_3)], vec![write(REF_4), write(REF_2)]),
        vec![delete(REF_1), delete(REF_3), write(REF_4), write(REF_2)]
    );
}
