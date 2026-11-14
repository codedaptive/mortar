//! protected_move_verdict: fast-forward, creation and an unchanged value pass; a rewind, a sideways move, a move
//! to an unrelated commit and a delete (of a present or an absent ref) are refused with Protected naming the ref,
//! its value, the asked value and the protector.

use gitio::Oid;

use super::{protected_move_verdict, ProtectedRef};
use crate::store::{OplogError, RefMove};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn move_to(new: &str) -> RefMove {
    RefMove {
        refname: "refs/heads/main".to_string(),
        old: None,
        new: oid(new),
    }
}

fn guard() -> ProtectedRef {
    ProtectedRef {
        refname: "refs/heads/main".to_string(),
        by: "land".to_string(),
    }
}

fn assert_protected(got: Result<(), OplogError>, current: Option<&str>, new: &str) {
    match got {
        Err(OplogError::Protected {
            refname,
            current: got_current,
            new: got_new,
            by,
        }) => {
            assert_eq!(refname, "refs/heads/main");
            assert_eq!(got_current, current.map(oid));
            assert_eq!(got_new, oid(new));
            assert_eq!(by, "land");
        }
        other => panic!("expected Protected, got {other:?}"),
    }
}

#[test]
fn fast_forward_passes() {
    let got = protected_move_verdict(&move_to(B), &guard(), Some(&oid(A)), Some(&oid(A)));
    assert!(got.is_ok(), "{got:?}");
}

#[test]
fn creation_of_an_absent_ref_passes() {
    let got = protected_move_verdict(&move_to(B), &guard(), None, None);
    assert!(got.is_ok(), "{got:?}");
}

#[test]
fn same_value_passes() {
    let got = protected_move_verdict(&move_to(A), &guard(), Some(&oid(A)), None);
    assert!(got.is_ok(), "{got:?}");
}

#[test]
fn rewind_to_an_ancestor_is_refused() {
    let got = protected_move_verdict(&move_to(B), &guard(), Some(&oid(A)), Some(&oid(B)));
    assert_protected(got, Some(A), B);
}

#[test]
fn sideways_move_is_refused() {
    let got = protected_move_verdict(&move_to(B), &guard(), Some(&oid(A)), Some(&oid(C)));
    assert_protected(got, Some(A), B);
}

#[test]
fn move_to_an_unrelated_commit_is_refused() {
    let got = protected_move_verdict(&move_to(B), &guard(), Some(&oid(A)), None);
    assert_protected(got, Some(A), B);
}

#[test]
fn delete_of_a_present_ref_is_refused() {
    let got = protected_move_verdict(&move_to(ZERO), &guard(), Some(&oid(A)), None);
    assert_protected(got, Some(A), ZERO);
}

#[test]
fn delete_of_an_absent_ref_is_refused() {
    let got = protected_move_verdict(&move_to(ZERO), &guard(), None, None);
    assert_protected(got, None, ZERO);
}
