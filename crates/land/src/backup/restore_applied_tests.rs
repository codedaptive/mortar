//! restore_applied: Done as success; Repeated, Failed and an apply error each refused with what they carry.

use super::{restore_applied, BackupError};
use gitio::Oid;
use oplog::{ApplyOutcome, OpPhase, OpRecord, OpResult, OplogError, RefMove};

/// The one helper building values only a constructor makes (Oid::parse).
fn record(phase: OpPhase, result: OpResult) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "land-restore-backup:h1:op-1".to_string(),
        kind: "land-restore-backup".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 5,
        phase,
        moves: vec![RefMove {
            refname: "refs/heads/zone/a".to_string(),
            old: None,
            new: Oid::parse("2222222222222222222222222222222222222222").unwrap(),
        }],
        result: Some(result),
    }
}

#[test]
fn done_is_success() {
    assert!(restore_applied(Ok(ApplyOutcome::Done(record(OpPhase::Done, OpResult::Moved)))).is_ok());
}

#[test]
fn repeated_is_refused_with_the_done_record() {
    match restore_applied(Ok(ApplyOutcome::Repeated(record(OpPhase::Done, OpResult::Moved)))) {
        Err(BackupError::OpRepeated(got)) => {
            assert_eq!(got.op_id, "op-1");
            assert_eq!(got.phase, OpPhase::Done);
            assert_eq!(got.result, Some(OpResult::Moved));
        }
        other => panic!("expected OpRepeated, got {other:?}"),
    }
}

#[test]
fn failed_is_refused_with_its_record() {
    let stale = OpResult::Stale {
        refname: "refs/heads/zone/a".to_string(),
        actual: Some(Oid::parse("1111111111111111111111111111111111111111").unwrap()),
    };
    match restore_applied(Ok(ApplyOutcome::Failed(record(OpPhase::Failed, stale.clone())))) {
        Err(BackupError::OpFailed(got)) => {
            assert_eq!(got.op_id, "op-1");
            assert_eq!(got.phase, OpPhase::Failed);
            assert_eq!(got.result, Some(stale));
        }
        other => panic!("expected OpFailed, got {other:?}"),
    }
}

#[test]
fn an_apply_error_is_oplog() {
    let refusal = OplogError::BadMove { refname: "refs/heads/zone/a".to_string(), reason: "bad".to_string() };
    match restore_applied(Err(refusal)) {
        Err(BackupError::Oplog(OplogError::BadMove { refname, reason })) => {
            assert_eq!(refname, "refs/heads/zone/a");
            assert_eq!(reason, "bad");
        }
        other => panic!("expected Oplog(BadMove), got {other:?}"),
    }
}
