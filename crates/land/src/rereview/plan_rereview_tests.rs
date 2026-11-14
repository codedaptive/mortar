//! plan_rereview: same tree and parents (roots included) is NothingChanged; a root is NoParent naming it, reviewed
//! first; one first parent is RangeDiff on it, also for a different tree; different first parents are BasesDiffer.

use gitio::{CommitInfo, Oid, Person};

use super::{plan_rereview, RereviewError, RereviewPlan};

const REVIEWED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const TREE_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PARENT_C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const PARENT_D: &str = "dddddddddddddddddddddddddddddddddddddddd";

fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

fn commit(tree: &str, parents: &[&str]) -> CommitInfo {
    CommitInfo {
        tree: oid(tree),
        parents: parents.iter().map(|p| oid(p)).collect(),
        author: Person {
            name: "Ada".to_string(),
            email: "ada@example.com".to_string(),
            date: Some("1700000000 +0000".to_string()),
        },
        message: "squash\n".to_string(),
    }
}

fn plan(old: &CommitInfo, now: &CommitInfo) -> Result<RereviewPlan, RereviewError> {
    plan_rereview(&oid(REVIEWED), old, &oid(NEW), now)
}

#[test]
fn same_tree_and_parents_is_nothing_changed() {
    let got = plan(&commit(TREE_A, &[PARENT_C]), &commit(TREE_A, &[PARENT_C])).unwrap();
    assert_eq!(got, RereviewPlan::NothingChanged);
}

#[test]
fn identical_roots_are_nothing_changed() {
    let got = plan(&commit(TREE_A, &[]), &commit(TREE_A, &[])).unwrap();
    assert_eq!(got, RereviewPlan::NothingChanged);
}

#[test]
fn reviewed_root_is_no_parent_naming_reviewed() {
    match plan(&commit(TREE_A, &[]), &commit(TREE_B, &[PARENT_C])) {
        Err(RereviewError::NoParent { commit }) => assert_eq!(commit, oid(REVIEWED)),
        other => panic!("gave {other:?}, not NoParent"),
    }
}

#[test]
fn new_root_is_no_parent_naming_new() {
    match plan(&commit(TREE_A, &[PARENT_C]), &commit(TREE_B, &[])) {
        Err(RereviewError::NoParent { commit }) => assert_eq!(commit, oid(NEW)),
        other => panic!("gave {other:?}, not NoParent"),
    }
}

#[test]
fn both_roots_with_different_trees_name_reviewed_first() {
    match plan(&commit(TREE_A, &[]), &commit(TREE_B, &[])) {
        Err(RereviewError::NoParent { commit }) => assert_eq!(commit, oid(REVIEWED)),
        other => panic!("gave {other:?}, not NoParent"),
    }
}

#[test]
fn different_tree_on_same_parent_is_range_diff_on_that_parent() {
    let got = plan(&commit(TREE_A, &[PARENT_C]), &commit(TREE_B, &[PARENT_C])).unwrap();
    assert_eq!(got, RereviewPlan::RangeDiff { base: oid(PARENT_C) });
}

#[test]
fn same_first_parent_with_other_parents_differing_is_range_diff_on_it() {
    let got = plan(&commit(TREE_A, &[PARENT_C, PARENT_D]), &commit(TREE_A, &[PARENT_C])).unwrap();
    assert_eq!(got, RereviewPlan::RangeDiff { base: oid(PARENT_C) });
}

#[test]
fn different_first_parents_are_bases_differ_naming_both() {
    match plan(&commit(TREE_A, &[PARENT_C]), &commit(TREE_A, &[PARENT_D])) {
        Err(RereviewError::BasesDiffer { reviewed_parent, new_parent }) => {
            assert_eq!(reviewed_parent, oid(PARENT_C));
            assert_eq!(new_parent, oid(PARENT_D));
        }
        other => panic!("gave {other:?}, not BasesDiffer"),
    }
}
