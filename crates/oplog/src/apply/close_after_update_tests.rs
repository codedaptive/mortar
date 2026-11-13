//! close_after_update: Updated closes done/Moved; Stale closes failed/Stale with the actual value.

use gitio::{Oid, UpdateRefsOutcome};

use super::close_after_update;
use crate::store::{OpPhase, OpRecord, OpResult, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn moves() -> Vec<RefMove> {
    vec![RefMove {
        refname: "refs/heads/landing".to_string(),
        old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    }]
}

fn record(at: u64, phase: OpPhase, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land:zone-a".to_string(),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "squash lands".to_string(),
        at,
        phase,
        moves: moves(),
        result,
    }
}

fn intent() -> OpRecord {
    record(100, OpPhase::Intent, None)
}

#[test]
fn updated_closes_done_moved() {
    assert_eq!(
        close_after_update(&intent(), &UpdateRefsOutcome::Updated, None, 105),
        record(105, OpPhase::Done, Some(OpResult::Moved))
    );
}

#[test]
fn updated_ignores_an_actual_value() {
    let actual = Some(oid("cccccccccccccccccccccccccccccccccccccccc"));
    assert_eq!(
        close_after_update(&intent(), &UpdateRefsOutcome::Updated, actual, 105),
        record(105, OpPhase::Done, Some(OpResult::Moved))
    );
}

#[test]
fn stale_closes_failed_with_the_actual_value() {
    let outcome = UpdateRefsOutcome::Stale {
        refname: "refs/heads/landing".to_string(),
    };
    let actual = Some(oid("cccccccccccccccccccccccccccccccccccccccc"));
    assert_eq!(
        close_after_update(&intent(), &outcome, actual, 106),
        record(
            106,
            OpPhase::Failed,
            Some(OpResult::Stale {
                refname: "refs/heads/landing".to_string(),
                actual: Some(oid("cccccccccccccccccccccccccccccccccccccccc")),
            })
        )
    );
}

#[test]
fn stale_on_an_absent_ref_closes_failed_with_no_actual_value() {
    let outcome = UpdateRefsOutcome::Stale {
        refname: "refs/heads/landing".to_string(),
    };
    assert_eq!(
        close_after_update(&intent(), &outcome, None, 107),
        record(
            107,
            OpPhase::Failed,
            Some(OpResult::Stale {
                refname: "refs/heads/landing".to_string(),
                actual: None,
            })
        )
    );
}
