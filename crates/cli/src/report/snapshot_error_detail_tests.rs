use std::path::PathBuf;

use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};
use snapshot::SnapshotError;

use super::snapshot_error_detail;

fn oid() -> Oid {
    Oid::parse("1111111111111111111111111111111111111111").unwrap()
}

fn git_error() -> GitError {
    GitError { args: vec!["read-tree".to_string(), "HEAD".to_string()], code: Some(128), stderr: "fatal: index.lock exists\n".to_string() }
}

fn record(result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(), key: "key-1".to_string(), kind: "save".to_string(),
        actor: "a/r1".to_string(), reason: "save work".to_string(), at: 1,
        phase: OpPhase::Failed, moves: vec![], result,
    }
}

#[test]
fn wrapped_errors_use_inner_display_without_snapshot_prefix() {
    let cases = [
        (SnapshotError::Git(git_error()), "git read-tree HEAD: exit code 128: fatal: index.lock exists"),
        (SnapshotError::Oplog(OplogError::Malformed { line: None, reason: "no op_id".to_string() }), "malformed op data: no op_id; mortar did not write this text, so check what did"),
        (SnapshotError::Trailers(MortarTrailerError::Git(git_error())), "git trailers failed: git read-tree HEAD: exit code 128: fatal: index.lock exists"),
        (SnapshotError::ChangeId(ChangeIdError::Malformed { text: "abc".to_string(), reason: "too short".to_string() }), "not a change ID: \"abc\": too short"),
    ];
    for (error, expected) in cases {
        assert_eq!(snapshot_error_detail(&error), expected);
    }
}

#[test]
fn missing_change_id_makes_no_global_movement_claim() {
    assert_eq!(snapshot_error_detail(&SnapshotError::ChangeIdMissing { head: oid() }),
        "HEAD 1111111111111111111111111111111111111111 is this request's and zone's but carries no Mortar-Change-Id");
}

#[test]
fn snapshot_ref_stale_reports_actual_or_absent_scoped_to_snapshot() {
    let held = SnapshotError::RefMoved(record(Some(OpResult::Stale { refname: "refs/snap/a".to_string(), actual: Some(oid()) })));
    let absent = SnapshotError::RefMoved(record(Some(OpResult::Stale { refname: "refs/snap/a".to_string(), actual: None })));
    assert_eq!(snapshot_error_detail(&held), "the snapshot ref refs/snap/a was not at the value read (it holds 1111111111111111111111111111111111111111); this snapshot moved nothing");
    assert_eq!(snapshot_error_detail(&absent), "the snapshot ref refs/snap/a was not at the value read (it holds absent); this snapshot moved nothing");
}

#[test]
fn snapshot_ref_without_stale_result_names_operation() {
    assert_eq!(snapshot_error_detail(&SnapshotError::RefMoved(record(Some(OpResult::Moved)))),
        "the snapshot ref was not at the value read (op op-1); this snapshot moved nothing");
}

#[test]
fn repeated_operation_scopes_movement_to_this_attempt() {
    assert_eq!(snapshot_error_detail(&SnapshotError::OpRepeated(record(Some(OpResult::Moved)))),
        "the op log already holds a done record for key key-1 (op op-1); this attempt moved nothing");
}

#[test]
fn file_size_failure_scopes_commit_claim_to_snapshot() {
    let error = SnapshotError::FileSize { path: PathBuf::from("data/loop"), error: std::io::Error::other("cannot stat") };
    assert_eq!(snapshot_error_detail(&error), "the size of data/loop could not be read: cannot stat; this snapshot wrote no commit");
}

#[test]
fn non_utf8_path_failure_scopes_commit_claim_to_snapshot() {
    let error = SnapshotError::PathNotUtf8 { path: PathBuf::from("data/big.bin") };
    assert_eq!(snapshot_error_detail(&error), "the refused file \"data/big.bin\" has a name that is not UTF-8, so it cannot be passed to git as an exclude; this snapshot wrote no commit");
}

#[test]
fn cap_mismatch_names_paths_and_scopes_commit_claim() {
    let error = SnapshotError::CapMismatch { paths: vec![PathBuf::from("a/x.bin"), PathBuf::from("b/y.bin")] };
    assert_eq!(snapshot_error_detail(&error), "the capped tree did not differ from the full tree by exactly the refused files: a/x.bin, b/y.bin; this snapshot wrote no commit");
}

#[test]
fn branch_stale_reports_actual_or_absent_without_movement_claim() {
    let held = SnapshotError::BranchMoved(record(Some(OpResult::Stale { refname: "refs/heads/a".to_string(), actual: Some(oid()) })));
    let absent = SnapshotError::BranchMoved(record(Some(OpResult::Stale { refname: "refs/heads/a".to_string(), actual: None })));
    assert_eq!(snapshot_error_detail(&held), "the branch refs/heads/a was not at the value read (it holds 1111111111111111111111111111111111111111)");
    assert_eq!(snapshot_error_detail(&absent), "the branch refs/heads/a was not at the value read (it holds absent)");
}

#[test]
fn branch_without_stale_result_names_operation() {
    assert_eq!(snapshot_error_detail(&SnapshotError::BranchMoved(record(None))), "the branch was not at the value read (op op-1)");
}

#[test]
fn remaining_errors_keep_snapshot_display() {
    let cases = [
        (SnapshotError::UnbornHead { folder: PathBuf::from("/w/a") }, "HEAD of the folder /w/a has no commit"),
        (SnapshotError::DetachedHead { folder: PathBuf::from("/w/a") }, "HEAD of the folder /w/a names no branch"),
        (SnapshotError::BadTarget { reason: "attempt 0".to_string() }, "the snapshot ref cannot be named: attempt 0"),
        (SnapshotError::BadSentence { reason: "empty".to_string() }, "the save sentence is refused: empty"),
    ];
    for (error, expected) in cases {
        assert_eq!(snapshot_error_detail(&error), expected);
    }
}

#[test]
fn index_not_set_scopes_the_untouched_files_to_the_index_step() {
    assert_eq!(
        snapshot_error_detail(&SnapshotError::IndexNotSet { commit: oid(), error: git_error() }),
        "the folder's index could not be set to commit 1111111111111111111111111111111111111111's tree: git read-tree HEAD: exit code 128: fatal: index.lock exists; the index step left the files untouched, and the index is the old one, or that tree without its skip-worktree bits if the sparse-checkout reapply failed"
    );
}
