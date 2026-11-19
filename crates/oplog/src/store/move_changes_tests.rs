//! move_changes: a create, an update and a delete change the ref; old equal to new and a delete of an absent ref
//! do not.

use gitio::Oid;

use super::move_changes;
use crate::store::RefMove;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn mv(old: Option<&str>, new: &str) -> RefMove {
    RefMove {
        refname: "refs/heads/main".to_string(),
        old: old.map(|o| Oid::parse(o).unwrap()),
        new: Oid::parse(new).unwrap(),
    }
}

#[test]
fn create_changes() {
    assert!(move_changes(&mv(None, B)));
}

#[test]
fn update_changes() {
    assert!(move_changes(&mv(Some(A), B)));
}

#[test]
fn delete_changes() {
    assert!(move_changes(&mv(Some(A), ZERO)));
}

#[test]
fn old_equal_to_new_does_not_change() {
    assert!(!move_changes(&mv(Some(A), A)));
}

#[test]
fn delete_of_absent_ref_does_not_change() {
    assert!(!move_changes(&mv(None, ZERO)));
}

#[test]
fn zero_old_to_zero_new_does_not_change() {
    // old == new, even when both are the all-zero value: a logged op may carry it.
    assert!(!move_changes(&mv(Some(ZERO), ZERO)));
}
