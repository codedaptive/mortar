//! is_zero_oid: all zeros in either hash length is the zero id; one other digit is not. ZUT: gitio's Oid::parse is a
//! constructor, used as data.

use gitio::Oid;

use super::is_zero_oid;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

#[test]
fn forty_zeros_is_zero() {
    assert!(is_zero_oid(&oid("0000000000000000000000000000000000000000")));
}

#[test]
fn sixty_four_zeros_is_zero() {
    assert!(is_zero_oid(&oid("0000000000000000000000000000000000000000000000000000000000000000")));
}

#[test]
fn one_non_zero_digit_is_not_zero() {
    assert!(!is_zero_oid(&oid("0000000000000000000000000000000000000001")));
}
