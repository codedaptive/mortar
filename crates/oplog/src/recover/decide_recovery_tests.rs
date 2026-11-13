//! decide_recovery: all at new is done, all at old is failed, anything else is mixed with the actual values.

use gitio::Oid;

use super::{decide_recovery, RecoveryDecision};
use crate::store::RefMove;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const D: &str = "dddddddddddddddddddddddddddddddddddddddd";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn mv(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove {
        refname: refname.to_string(),
        old: old.map(oid),
        new: oid(new),
    }
}

#[test]
fn every_ref_at_new_is_done() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), Some(oid(B))),
        (mv("refs/heads/main", Some(C), D), Some(oid(D))),
    ];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}

#[test]
fn every_ref_at_old_is_failed() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), Some(oid(A))),
        (mv("refs/heads/main", Some(C), D), Some(oid(C))),
    ];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Failed);
}

#[test]
fn one_at_new_one_at_old_is_mixed_with_actual_values() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), Some(oid(B))),
        (mv("refs/heads/main", Some(C), D), Some(oid(C))),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Mixed(vec![
            (mv("refs/heads/landing", Some(A), B), Some(oid(B))),
            (mv("refs/heads/main", Some(C), D), Some(oid(C))),
        ])
    );
}

#[test]
fn a_ref_at_neither_value_is_mixed() {
    let refs = vec![(mv("refs/heads/landing", Some(A), B), Some(oid(C)))];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Mixed(vec![(mv("refs/heads/landing", Some(A), B), Some(oid(C)))])
    );
}

#[test]
fn a_ref_absent_with_an_old_value_is_mixed() {
    let refs = vec![(mv("refs/heads/landing", Some(A), B), None)];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Mixed(vec![(mv("refs/heads/landing", Some(A), B), None)])
    );
}

#[test]
fn a_delete_whose_ref_is_absent_is_done() {
    let refs = vec![(mv("refs/heads/topic", Some(A), ZERO), None)];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}

#[test]
fn a_delete_whose_ref_still_holds_old_is_failed() {
    let refs = vec![(mv("refs/heads/topic", Some(A), ZERO), Some(oid(A)))];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Failed);
}

#[test]
fn a_create_whose_ref_is_absent_is_failed() {
    let refs = vec![(mv("refs/heads/topic", None, B), None)];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Failed);
}

#[test]
fn a_create_whose_ref_holds_new_is_done() {
    let refs = vec![(mv("refs/heads/topic", None, B), Some(oid(B)))];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}

#[test]
fn no_moves_is_done() {
    assert_eq!(decide_recovery(&[]), RecoveryDecision::Done);
}

#[test]
fn old_equal_to_new_is_done() {
    let refs = vec![(mv("refs/heads/landing", Some(A), A), Some(oid(A)))];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}
