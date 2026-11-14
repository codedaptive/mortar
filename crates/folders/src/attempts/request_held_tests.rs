//! request_held: true only when the merge base is the request tip itself.

use gitio::Oid;

use super::request_held;

const REQ: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn held_when_the_merge_base_is_the_request_tip() {
    assert!(request_held(&oid(REQ), Some(&oid(REQ))));
}

#[test]
fn not_held_when_the_merge_base_is_another_commit() {
    assert!(!request_held(&oid(REQ), Some(&oid(OTHER))));
}

#[test]
fn not_held_with_no_merge_base() {
    assert!(!request_held(&oid(REQ), None));
}
