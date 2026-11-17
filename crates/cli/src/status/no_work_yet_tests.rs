//! no_work_yet: a folder has no work yet when its branch tip is still its record's base and its tree equals the
//! tip's tree (S-145); commits of its own, or files that differ from the tip, are work.

use gitio::Oid;

use super::no_work_yet;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

const BASE: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";
const TIP_TREE: &str = "3333333333333333333333333333333333333333";
const OTHER_TREE: &str = "4444444444444444444444444444444444444444";

#[test]
fn a_branch_at_its_base_with_the_folder_at_the_tip_tree_has_no_work_yet() {
    assert!(no_work_yet(&oid(BASE), &oid(BASE), &oid(TIP_TREE), &oid(TIP_TREE)));
}

#[test]
fn a_branch_at_its_base_with_the_folder_tree_other_than_the_tip_tree_has_work() {
    assert!(!no_work_yet(&oid(BASE), &oid(BASE), &oid(OTHER_TREE), &oid(TIP_TREE)));
}

#[test]
fn a_branch_past_its_base_has_work_even_with_the_folder_at_the_tip_tree() {
    assert!(!no_work_yet(&oid(BASE), &oid(TIP), &oid(TIP_TREE), &oid(TIP_TREE)));
}

#[test]
fn a_branch_past_its_base_with_the_folder_tree_other_than_the_tip_tree_has_work() {
    assert!(!no_work_yet(&oid(BASE), &oid(TIP), &oid(OTHER_TREE), &oid(TIP_TREE)));
}
