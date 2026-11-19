use crate::save_from_apply::save_from_apply;
use crate::SnapshotError;
use gitio::Oid;
use oplog::{ApplyOutcome, OpPhase, OpRecord, OpResult, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(phase: OpPhase, result: OpResult) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "save refs/heads/zone/snapshot op-1".to_string(),
        kind: "save".to_string(),
        actor: "agent-a".to_string(),
        reason: "save".to_string(),
        at: 1700000000,
        phase,
        moves: vec![RefMove {
            refname: "refs/heads/zone/snapshot".to_string(),
            old: Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")),
            new: oid("dddddddddddddddddddddddddddddddddddddddd"),
        }],
        result: Some(result),
    }
}

#[test]
fn done_is_ok() {
    assert!(save_from_apply(ApplyOutcome::Done(record(OpPhase::Done, OpResult::Moved))).is_ok());
}

#[test]
fn repeated_is_op_repeated_with_the_earlier_done_record() {
    let done = record(OpPhase::Done, OpResult::Moved);
    match save_from_apply(ApplyOutcome::Repeated(done.clone())) {
        Err(SnapshotError::OpRepeated(carried)) => assert_eq!(carried, done),
        other => panic!("expected OpRepeated, got {other:?}"),
    }
}

#[test]
fn failed_is_branch_moved_with_the_failed_record() {
    let failed = record(
        OpPhase::Failed,
        OpResult::Stale {
            refname: "refs/heads/zone/snapshot".to_string(),
            actual: Some(oid("ffffffffffffffffffffffffffffffffffffffff")),
        },
    );
    match save_from_apply(ApplyOutcome::Failed(failed.clone())) {
        Err(SnapshotError::BranchMoved(carried)) => assert_eq!(carried, failed),
        other => panic!("expected BranchMoved, got {other:?}"),
    }
}
