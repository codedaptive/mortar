//! joined_leftovers: a leftover branch is kept only when its merge base with its zone tip is its own tip.

use gitio::Oid;

use super::joined_leftovers::joined_leftovers;
use super::Leftover;

const ZONE: &str = "1111111111111111111111111111111111111111";
const TIP_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TIP_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const TIP_C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const TIP_D: &str = "dddddddddddddddddddddddddddddddddddddddd";
const FORK: &str = "0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn leftover(request: &str, tip: &str) -> Leftover {
    Leftover { branch: format!("refs/heads/req/folders/{request}"), tip: oid(tip), zone_tip: oid(ZONE) }
}

#[test]
fn a_branch_whose_merge_base_is_its_tip_is_kept() {
    assert_eq!(
        joined_leftovers(vec![(leftover("WO-1", TIP_A), Some(oid(TIP_A)))]),
        vec![leftover("WO-1", TIP_A)]
    );
}

#[test]
fn a_branch_whose_merge_base_is_another_commit_is_dropped() {
    // The zone moved past the commit the branch forked from; the branch's own commits never joined.
    assert_eq!(joined_leftovers(vec![(leftover("WO-1", TIP_A), Some(oid(FORK)))]), Vec::<Leftover>::new());
}

#[test]
fn a_branch_with_no_merge_base_is_dropped() {
    assert_eq!(joined_leftovers(vec![(leftover("WO-1", TIP_A), None)]), Vec::<Leftover>::new());
}

#[test]
fn a_mix_keeps_exactly_the_contained_branches_in_input_order() {
    assert_eq!(
        joined_leftovers(vec![
            (leftover("WO-4", TIP_D), Some(oid(TIP_D))),
            (leftover("WO-2", TIP_B), Some(oid(FORK))),
            (leftover("WO-1", TIP_A), Some(oid(TIP_A))),
            (leftover("WO-3", TIP_C), None),
        ]),
        vec![leftover("WO-4", TIP_D), leftover("WO-1", TIP_A)]
    );
}

#[test]
fn no_candidates_give_no_leftovers() {
    assert_eq!(joined_leftovers(Vec::new()), Vec::<Leftover>::new());
}
