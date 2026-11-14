//! WO-orc-snapshot-rit-1 #1453, S-101: the eight cross-kit SnapshotError Display cases removed from units.
//! SnapshotError::fmt -> real Gitio/Oplog Display, with literal expected contracts, not inner.to_string or fakes.
//! Every parent owns a disposable TestRepo; its init isolates global/system Git config (#1419).
use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError, RefMove};
use snapshot::SnapshotError;
use testkit::TestRepo;

const HEX: &str = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
fn oid() -> Oid { Oid::parse(HEX).unwrap() }
fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no\n".into() }
}
fn stale_record() -> OpRecord {
    OpRecord {
        op_id: "op-7".into(), key: "save refs/heads/x op-7".into(), kind: "save".into(), actor: "bilby".into(),
        reason: "save".into(), at: 1700000000, phase: OpPhase::Failed,
        moves: vec![RefMove { refname: "refs/heads/x".into(), old: None, new: oid() }],
        result: Some(OpResult::Stale { refname: "refs/heads/x".into(), actual: Some(oid()) }),
    }
}

#[test]
fn rit_git_variant_preserves_gitios_command_exit_and_trimmed_stderr_after_snapshot_prefix() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(SnapshotError::Git(git_error()).to_string(),
        "a git run failed: git read-tree HEAD: exit code 128: fatal: no");
}

#[test]
fn rit_oplog_variant_keeps_oplogs_complete_malformed_diagnostic_after_snapshot_prefix() {
    let _repo = TestRepo::new().unwrap();
    let inner = OplogError::Malformed { line: None, reason: "no op_id".into() };
    assert_eq!(SnapshotError::Oplog(inner).to_string(),
        "the op log refused: malformed op data: no op_id; mortar did not write this text, so check what did");
}

#[test]
fn rit_change_id_missing_formats_the_real_gitio_head_oid_and_exact_no_write_no_move_account() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(SnapshotError::ChangeIdMissing { head: oid() }.to_string(),
        "HEAD abcdefabcdefabcdefabcdefabcdefabcdefabcd is this request's and zone's but carries no Mortar-Change-Id; no commit was written and no branch moved");
}

#[test]
fn rit_trailers_variant_keeps_both_oplog_and_gitio_error_layers_after_snapshot_prefix() {
    let _repo = TestRepo::new().unwrap();
    let inner = MortarTrailerError::Git(git_error());
    assert_eq!(SnapshotError::Trailers(inner).to_string(),
        "HEAD's trailers could not be read: git trailers failed: git read-tree HEAD: exit code 128: fatal: no");
}

#[test]
fn rit_change_id_variant_keeps_oplogs_quoted_bad_text_and_reason_after_snapshot_prefix() {
    let _repo = TestRepo::new().unwrap();
    let inner = ChangeIdError::Malformed { text: "abc".into(), reason: "too short".into() };
    assert_eq!(SnapshotError::ChangeId(inner).to_string(),
        "a new change ID could not be drawn: not a change ID: \"abc\": too short");
}

#[test]
fn rit_ref_moved_formats_oplogs_stale_ref_and_actual_gitio_oid_without_losing_the_no_move_account() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(SnapshotError::RefMoved(stale_record()).to_string(),
        "the snapshot ref refs/heads/x was not at the value read (it holds abcdefabcdefabcdefabcdefabcdefabcdefabcd); nothing moved");
}

#[test]
fn rit_branch_moved_formats_oplogs_stale_ref_and_actual_gitio_oid_with_the_branch_specific_account() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(SnapshotError::BranchMoved(stale_record()).to_string(),
        "the branch refs/heads/x was not at the value read (it holds abcdefabcdefabcdefabcdefabcdefabcdefabcd); the branch did not move");
}

#[test]
fn rit_index_not_set_formats_commit_oid_and_gitio_error_with_the_exact_preservation_account() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(SnapshotError::IndexNotSet { commit: oid(), error: git_error() }.to_string(),
        "the folder's index could not be set to commit abcdefabcdefabcdefabcdefabcdefabcdefabcd's tree: git read-tree HEAD: exit code 128: fatal: no; the files are untouched and the index is the old one");
}
