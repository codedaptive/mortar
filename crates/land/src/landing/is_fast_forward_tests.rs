//! is_fast_forward: one parent at integration is a fast-forward; another parent, none, or two (even one at
//! integration) are not.

use super::is_fast_forward;
use gitio::Oid;

const INTEGRATION: &str = "1111111111111111111111111111111111111111";
const OTHER: &str = "2222222222222222222222222222222222222222";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn one_parent_at_integration_is_a_fast_forward() {
    assert!(is_fast_forward(&[oid(INTEGRATION)], &oid(INTEGRATION)));
}

#[test]
fn one_parent_elsewhere_is_not() {
    assert!(!is_fast_forward(&[oid(OTHER)], &oid(INTEGRATION)));
}

#[test]
fn no_parent_is_not() {
    assert!(!is_fast_forward(&[], &oid(INTEGRATION)));
}

#[test]
fn two_parents_are_not_even_with_one_at_integration() {
    assert!(!is_fast_forward(&[oid(INTEGRATION), oid(OTHER)], &oid(INTEGRATION)));
    assert!(!is_fast_forward(&[oid(OTHER), oid(INTEGRATION)], &oid(INTEGRATION)));
}
