//! oid_is_zero: true for the 40- and 64-zero ids, false when any character is not '0'.

use gitio::Oid;

use super::oid_is_zero;

#[test]
fn forty_zeros_is_zero() {
    let oid = Oid::parse("0000000000000000000000000000000000000000").unwrap();
    assert!(oid_is_zero(&oid));
}

#[test]
fn sixty_four_zeros_is_zero() {
    let oid = Oid::parse("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    assert!(oid_is_zero(&oid));
}

#[test]
fn one_non_zero_character_is_not_zero() {
    let oid = Oid::parse("0000000000000000000000000000000000000001").unwrap();
    assert!(!oid_is_zero(&oid));
}
