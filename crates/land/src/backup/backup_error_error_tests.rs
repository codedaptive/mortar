//! source(): the wrapped error for Host, Git and Oplog; none for the refusals backup and restore make themselves.

use super::BackupError;
use gitio::{GitError, PushRefusal};
use oplog::{LockError, OpPhase, OpRecord, OplogError};
use std::error::Error;

fn git_error() -> GitError {
    GitError { args: vec!["push".to_string()], code: Some(1), stderr: "bad".to_string() }
}

fn record() -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "k".to_string(),
        kind: "land-restore-backup".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 1,
        phase: OpPhase::Failed,
        moves: Vec::new(),
        result: None,
    }
}

#[test]
fn host_source_is_the_inner_lock_error() {
    let error = BackupError::Host(LockError::HostUnknown("no name".to_string()));
    let reason = error.source().and_then(|s| s.downcast_ref::<LockError>()).map(|e| match e {
        LockError::HostUnknown(reason) => reason.as_str(),
        _ => "another variant",
    });
    assert_eq!(reason, Some("no name"));
}

#[test]
fn git_source_is_the_inner_git_error() {
    let error = BackupError::Git(git_error());
    assert_eq!(error.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&git_error()));
}

#[test]
fn oplog_source_is_the_inner_oplog_error() {
    let error = BackupError::Oplog(OplogError::UnknownOp { op_id: "op-3".to_string() });
    let op_id = error.source().and_then(|s| s.downcast_ref::<OplogError>()).map(|e| match e {
        OplogError::UnknownOp { op_id } => op_id.as_str(),
        _ => "another variant",
    });
    assert_eq!(op_id, Some("op-3"));
}

#[test]
fn refusals_backup_makes_itself_have_no_source() {
    let refusals = [
        BackupError::BadHost(String::new()),
        BackupError::Refused { refname: "refs/mortar-backup/h1/heads/zone/a".to_string(), refusal: PushRefusal::Hook },
        BackupError::OpFailed(record()),
        BackupError::OpRepeated(record()),
    ];
    for refusal in &refusals {
        assert!(refusal.source().is_none(), "{refusal:?}");
    }
}
