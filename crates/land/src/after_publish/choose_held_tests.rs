//! choose_held: a held record wins, also over a held source; a record not held falls back to a held source, and to
//! None when the source is missing; without a record a held source is held, and a source not held or missing is
//! None.

use super::choose_held;
use gitio::Oid;

const SOURCE: &str = "1111111111111111111111111111111111111111";
const RECORDED: &str = "2222222222222222222222222222222222222222";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn a_record_the_tip_holds_is_held() {
    assert_eq!(choose_held(&oid(SOURCE), Some((oid(RECORDED), true)), Some(false)), Some(oid(RECORDED)));
}

#[test]
fn a_held_record_wins_over_a_held_source() {
    assert_eq!(choose_held(&oid(SOURCE), Some((oid(RECORDED), true)), Some(true)), Some(oid(RECORDED)));
}

#[test]
fn a_record_not_held_falls_back_to_a_held_source() {
    assert_eq!(choose_held(&oid(SOURCE), Some((oid(RECORDED), false)), Some(true)), Some(oid(SOURCE)));
}

#[test]
fn a_record_not_held_and_a_missing_source_is_none() {
    assert_eq!(choose_held(&oid(SOURCE), Some((oid(RECORDED), false)), None), None);
}

#[test]
fn no_record_and_a_held_source_is_the_source() {
    assert_eq!(choose_held(&oid(SOURCE), None, Some(true)), Some(oid(SOURCE)));
}

#[test]
fn no_record_and_a_source_not_held_is_none() {
    assert_eq!(choose_held(&oid(SOURCE), None, Some(false)), None);
}

#[test]
fn no_record_and_a_missing_source_is_none() {
    assert_eq!(choose_held(&oid(SOURCE), None, None), None);
}
