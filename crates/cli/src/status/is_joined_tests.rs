//! is_joined: a request is joined when the zone holds its tip and it has commits of its own.

use gitio::Oid;

use super::is_joined;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

const BASE: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";

#[test]
fn a_tip_past_the_base_held_by_the_zone_is_joined() {
    assert!(is_joined(&oid(BASE), &oid(TIP), true));
}

#[test]
fn a_tip_the_zone_does_not_hold_is_not_joined() {
    assert!(!is_joined(&oid(BASE), &oid(TIP), false));
}

#[test]
fn a_request_still_at_its_base_is_not_joined() {
    assert!(!is_joined(&oid(BASE), &oid(BASE), true));
    assert!(!is_joined(&oid(BASE), &oid(BASE), false));
}
