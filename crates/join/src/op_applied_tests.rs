//! op_applied: Done as success; Repeated, Failed and an apply error each refused with what they carry.

use crate::op_applied::op_applied;
use crate::JoinRunError;
use gitio::Oid;
use oplog::{ApplyOutcome, OpPhase, OpRecord, OpResult, OplogError, RefMove};

fn record(phase: OpPhase, result: OpResult) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "join-fast-forward:refs/heads/zone/join:a:b".to_string(),
        kind: "join-fast-forward".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 5,
        phase,
        moves: vec![RefMove {
            refname: "refs/heads/zone/join".to_string(),
            old: Some(Oid::parse("1111111111111111111111111111111111111111").unwrap()),
            new: Oid::parse("2222222222222222222222222222222222222222").unwrap(),
        }],
        result: Some(result),
    }
}

#[test]
fn done_is_success() {
    assert!(op_applied(Ok(ApplyOutcome::Done(record(OpPhase::Done, OpResult::Moved)))).is_ok());
}

#[test]
fn repeated_is_refused_with_the_done_record() {
    match op_applied(Ok(ApplyOutcome::Repeated(record(OpPhase::Done, OpResult::Moved)))) {
        Err(JoinRunError::OpRepeated(got)) => {
            assert_eq!(got.op_id, "op-1");
            assert_eq!(got.key, "join-fast-forward:refs/heads/zone/join:a:b");
            assert_eq!(got.phase, OpPhase::Done);
            assert_eq!(got.result, Some(OpResult::Moved));
        }
        other => panic!("expected OpRepeated, got {other:?}"),
    }
}

#[test]
fn failed_is_refused_with_its_record() {
    let stale = OpResult::Stale { refname: "refs/heads/zone/join".to_string(), actual: None };
    match op_applied(Ok(ApplyOutcome::Failed(record(OpPhase::Failed, stale.clone())))) {
        Err(JoinRunError::OpFailed(got)) => {
            assert_eq!(got.op_id, "op-1");
            assert_eq!(got.phase, OpPhase::Failed);
            assert_eq!(got.result, Some(stale));
        }
        other => panic!("expected OpFailed, got {other:?}"),
    }
}

#[test]
fn protected_refusal_is_oplog() {
    let refusal = OplogError::Protected {
        refname: "refs/heads/zone/join".to_string(),
        current: None,
        new: Oid::parse("2222222222222222222222222222222222222222").unwrap(),
        by: "join".to_string(),
    };
    match op_applied(Err(refusal)) {
        Err(JoinRunError::Oplog(OplogError::Protected { refname, by, .. })) => {
            assert_eq!(refname, "refs/heads/zone/join");
            assert_eq!(by, "join");
        }
        other => panic!("expected Oplog(Protected), got {other:?}"),
    }
}
