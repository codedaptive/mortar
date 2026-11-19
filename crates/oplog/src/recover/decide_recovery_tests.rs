//! decide_recovery: unchanged moves never decide; every changing move at new (or none to check) is done; at
//! least one changing move and none at new is failed; some at new and some not is mixed. Failed and mixed
//! carry each move with its actual value.

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
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![
            (mv("refs/heads/landing", Some(A), B), Some(oid(A))),
            (mv("refs/heads/main", Some(C), D), Some(oid(C))),
        ])
    );
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
fn a_ref_at_neither_value_with_none_at_new_is_failed() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), Some(oid(C))),
        (mv("refs/heads/main", Some(C), D), Some(oid(C))),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![
            (mv("refs/heads/landing", Some(A), B), Some(oid(C))),
            (mv("refs/heads/main", Some(C), D), Some(oid(C))),
        ])
    );
}

#[test]
fn refs_absent_with_old_values_are_failed() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), None),
        (mv("refs/heads/main", Some(C), D), None),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![
            (mv("refs/heads/landing", Some(A), B), None),
            (mv("refs/heads/main", Some(C), D), None),
        ])
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
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![(mv("refs/heads/topic", Some(A), ZERO), Some(oid(A)))])
    );
}

#[test]
fn a_create_whose_ref_is_absent_is_failed() {
    let refs = vec![(mv("refs/heads/topic", None, B), None)];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![(mv("refs/heads/topic", None, B), None)])
    );
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
fn an_unchanged_move_alone_is_done() {
    let refs = vec![(mv("refs/heads/landing", Some(A), A), Some(oid(A)))];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}

#[test]
fn an_unchanged_move_beside_a_ref_at_a_third_value_is_failed() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), A), Some(oid(A))),
        (mv("refs/heads/main", Some(C), D), Some(oid(B))),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![
            (mv("refs/heads/landing", Some(A), A), Some(oid(A))),
            (mv("refs/heads/main", Some(C), D), Some(oid(B))),
        ])
    );
}

#[test]
fn a_delete_of_an_absent_ref_beside_a_ref_at_a_third_value_is_failed() {
    let refs = vec![
        (mv("refs/heads/topic", None, ZERO), None),
        (mv("refs/heads/main", Some(C), D), Some(oid(B))),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![
            (mv("refs/heads/topic", None, ZERO), None),
            (mv("refs/heads/main", Some(C), D), Some(oid(B))),
        ])
    );
}

#[test]
fn a_changing_ref_at_new_beside_an_unchanged_move_and_a_ref_at_old_is_mixed() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), A), Some(oid(A))),
        (mv("refs/heads/main", Some(C), D), Some(oid(D))),
        (mv("refs/heads/topic", Some(B), C), Some(oid(B))),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Mixed(vec![
            (mv("refs/heads/landing", Some(A), A), Some(oid(A))),
            (mv("refs/heads/main", Some(C), D), Some(oid(D))),
            (mv("refs/heads/topic", Some(B), C), Some(oid(B))),
        ])
    );
}

#[test]
fn case_1_changing_move_at_new_beside_unchanged_ref_moved_elsewhere_is_done() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), Some(oid(B))),
        (mv("refs/heads/main", Some(C), C), Some(oid(D))),
    ];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}

#[test]
fn case_2_only_an_unchanged_ref_moved_elsewhere_is_done() {
    let refs = vec![(mv("refs/heads/main", Some(C), C), Some(oid(D)))];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}

#[test]
fn worked_example_changing_move_at_old_beside_unchanged_ref_is_failed() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), Some(oid(A))),
        (mv("refs/heads/main", Some(C), C), Some(oid(C))),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![
            (mv("refs/heads/landing", Some(A), B), Some(oid(A))),
            (mv("refs/heads/main", Some(C), C), Some(oid(C))),
        ])
    );
}

#[test]
fn worked_example_changing_move_still_at_old_beside_unchanged_ref_moved_elsewhere_is_failed() {
    let refs = vec![
        (mv("refs/heads/landing", Some(A), B), Some(oid(A))),
        (mv("refs/heads/main", Some(C), C), Some(oid(D))),
    ];
    assert_eq!(
        decide_recovery(&refs),
        RecoveryDecision::Failed(vec![
            (mv("refs/heads/landing", Some(A), B), Some(oid(A))),
            (mv("refs/heads/main", Some(C), C), Some(oid(D))),
        ])
    );
}

#[test]
fn an_unchanged_delete_whose_ref_now_holds_an_oid_beside_a_changing_move_at_new_is_done() {
    let refs = vec![
        (mv("refs/heads/topic", None, ZERO), Some(oid(D))),
        (mv("refs/heads/landing", Some(A), B), Some(oid(B))),
    ];
    assert_eq!(decide_recovery(&refs), RecoveryDecision::Done);
}
