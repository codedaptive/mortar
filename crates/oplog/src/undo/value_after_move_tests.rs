//! value_after_move: an all-zero new leaves the ref absent; any other new is the ref's value.

use gitio::Oid;

use super::value_after_move;

#[test]
fn zero_new_is_absent() {
    let zero = Oid::parse("0000000000000000000000000000000000000000").unwrap();
    assert_eq!(value_after_move(&zero), None);
}

#[test]
fn other_new_is_the_value() {
    let new = Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    assert_eq!(
        value_after_move(&new),
        Some(Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap())
    );
}
