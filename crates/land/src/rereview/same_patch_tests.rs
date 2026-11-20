//! same_patch: NothingChanged is the same patch whatever the IDs; a RangeDiff is the same patch only when both IDs
//! are present and equal; an empty change (None) on either side never is.

use gitio::{Oid, PatchId};

use super::{same_patch, RereviewPlan};

const PATCH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PATCH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const REVIEWED_BASE: &str = "1111111111111111111111111111111111111111";
const NEW_BASE: &str = "2222222222222222222222222222222222222222";

fn patch(text: &str) -> PatchId {
    PatchId::parse(text).unwrap()
}

fn range_diff() -> RereviewPlan {
    RereviewPlan::RangeDiff {
        reviewed_base: Oid::parse(REVIEWED_BASE).unwrap(),
        new_base: Oid::parse(NEW_BASE).unwrap(),
    }
}

#[test]
fn nothing_changed_is_same_with_different_ids() {
    assert!(same_patch(&RereviewPlan::NothingChanged, Some(&patch(PATCH_A)), Some(&patch(PATCH_B))));
}

#[test]
fn nothing_changed_is_same_with_no_ids() {
    assert!(same_patch(&RereviewPlan::NothingChanged, None, None));
}

#[test]
fn range_diff_with_equal_ids_is_same() {
    assert!(same_patch(&range_diff(), Some(&patch(PATCH_A)), Some(&patch(PATCH_A))));
}

#[test]
fn range_diff_with_different_ids_is_not_same() {
    assert!(!same_patch(&range_diff(), Some(&patch(PATCH_A)), Some(&patch(PATCH_B))));
}

#[test]
fn range_diff_with_reviewed_empty_is_not_same() {
    assert!(!same_patch(&range_diff(), None, Some(&patch(PATCH_A))));
}

#[test]
fn range_diff_with_new_empty_is_not_same() {
    assert!(!same_patch(&range_diff(), Some(&patch(PATCH_A)), None));
}

#[test]
fn range_diff_with_both_empty_is_not_same() {
    assert!(!same_patch(&range_diff(), None, None));
}
