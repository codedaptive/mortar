//! is_joined: a request is joined when the zone holds its tip, it has commits of its own, and the folder's tree is
//! the tip's tree.

use gitio::Oid;

use super::is_joined;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

const BASE: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";
const TIP_TREE: &str = "3333333333333333333333333333333333333333";
const OTHER_TREE: &str = "4444444444444444444444444444444444444444";

#[test]
fn a_tip_past_the_base_held_by_the_zone_with_the_folder_at_its_tree_is_joined() {
    assert!(is_joined(&oid(BASE), &oid(TIP), true, &oid(TIP_TREE), &oid(TIP_TREE)));
}

#[test]
fn a_folder_tree_other_than_the_tip_tree_is_not_joined() {
    assert!(!is_joined(&oid(BASE), &oid(TIP), true, &oid(OTHER_TREE), &oid(TIP_TREE)));
}

#[test]
fn a_tip_the_zone_does_not_hold_is_not_joined() {
    assert!(!is_joined(&oid(BASE), &oid(TIP), false, &oid(TIP_TREE), &oid(TIP_TREE)));
}

#[test]
fn a_request_still_at_its_base_is_not_joined() {
    assert!(!is_joined(&oid(BASE), &oid(BASE), true, &oid(TIP_TREE), &oid(TIP_TREE)));
    assert!(!is_joined(&oid(BASE), &oid(BASE), false, &oid(TIP_TREE), &oid(TIP_TREE)));
}
