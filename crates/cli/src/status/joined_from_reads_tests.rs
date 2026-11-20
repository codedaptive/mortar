//! joined_from_reads: joined when the zone holds the tip, the tip is past the base and the folder's tree is the
//! tip's tree.

use gitio::Oid;
use snapshot::SavedState;

use super::{joined_from_reads, BranchReads};

const BASE: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";
const TIP_TREE: &str = "3333333333333333333333333333333333333333";
const OTHER_TREE: &str = "4444444444444444444444444444444444444444";
const ZONE_TIP: &str = "5555555555555555555555555555555555555555";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn reads(branch_tip: &str, request_in_zone: bool, folder_tree: &str) -> BranchReads {
    BranchReads {
        branch_ref: "refs/heads/req/core/fix-1".to_string(),
        branch_tip: oid(branch_tip),
        zone_tip: oid(ZONE_TIP),
        request_in_zone,
        saved: SavedState { saved: true, tree: oid(folder_tree), refused: Vec::new() },
        tip_tree: oid(TIP_TREE),
    }
}

#[test]
fn a_tip_in_the_zone_past_the_base_with_the_folder_at_its_tree_is_joined() {
    assert!(joined_from_reads(&oid(BASE), &reads(TIP, true, TIP_TREE)));
}

#[test]
fn a_folder_tree_other_than_the_tip_tree_is_not_joined() {
    assert!(!joined_from_reads(&oid(BASE), &reads(TIP, true, OTHER_TREE)));
}

#[test]
fn a_tip_the_zone_does_not_hold_is_not_joined() {
    assert!(!joined_from_reads(&oid(BASE), &reads(TIP, false, TIP_TREE)));
}

#[test]
fn a_tip_still_at_the_base_is_not_joined() {
    assert!(!joined_from_reads(&oid(BASE), &reads(BASE, true, TIP_TREE)));
}
