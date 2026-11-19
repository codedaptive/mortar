use gitio::GitError;
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};
use snapshot::SnapshotError;

use super::snapshot_step_failure;
use crate::report::{ExitCode, Report};

fn git_error() -> GitError {
    GitError {
        args: vec!["read-tree".to_string(), "HEAD".to_string()],
        code: Some(128),
        stderr: "fatal: index.lock exists\n".to_string(),
    }
}

fn expected_report(verb: &str, detail: &str, left: &str, repeat: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        owner: None,
        next: "repeat the same command".to_string(),
        fields: vec![],
        lines: vec![detail.to_string(), left.to_string(), repeat.to_string()],
    }
}

#[test]
fn git_failure_for_join_describes_the_possible_snapshot_move() {
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::Git(git_error()), None),
        expected_report(
            "join",
            "git read-tree HEAD: exit code 128: fatal: index.lock exists",
            "the branch did not move; the snapshot ref may have moved",
            "a repeat finishes the join once the failure above is fixed",
        )
    );
}

#[test]
fn git_failure_for_resolve_describes_the_possible_snapshot_move() {
    assert_eq!(
        snapshot_step_failure("resolve", &SnapshotError::Git(git_error()), None),
        expected_report(
            "resolve",
            "git read-tree HEAD: exit code 128: fatal: index.lock exists",
            "the branch did not move; the snapshot ref may have moved",
            "a repeat finishes the resolve once the failure above is fixed",
        )
    );
}

#[test]
fn oplog_failure_for_join_describes_the_possible_open_op() {
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::Oplog(OplogError::Malformed {
            line: None, reason: "no op_id".to_string(),
        }), None),
        expected_report(
            "join",
            "malformed op data: no op_id; mortar did not write this text, so check what did",
            "the snapshot ref and the branch may have moved, and an op may be left open; the next command that takes the join lock settles it",
            "a repeat finishes the join once the failure above is fixed",
        )
    );
}

#[test]
fn oplog_failure_for_resolve_describes_the_possible_open_op() {
    assert_eq!(
        snapshot_step_failure("resolve", &SnapshotError::Oplog(OplogError::Malformed {
            line: None, reason: "no op_id".to_string(),
        }), None),
        expected_report(
            "resolve",
            "malformed op data: no op_id; mortar did not write this text, so check what did",
            "the snapshot ref and the branch may have moved, and an op may be left open; the next command that takes the join lock settles it",
            "a repeat finishes the resolve once the failure above is fixed",
        )
    );
}

#[test]
fn trailers_failure_for_join_describes_the_possible_snapshot_move() {
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::Trailers(MortarTrailerError::Git(git_error())), None),
        expected_report(
            "join",
            "git trailers failed: git read-tree HEAD: exit code 128: fatal: index.lock exists",
            "no save commit was written and the branch did not move; the snapshot ref may have moved",
            "a repeat finishes the join once the failure above is fixed",
        )
    );
}

#[test]
fn change_id_failure_for_join_describes_the_possible_snapshot_move() {
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::ChangeId(ChangeIdError::Malformed {
            text: "abc".to_string(), reason: "too short".to_string(),
        }), None),
        expected_report(
            "join",
            "not a change ID: \"abc\": too short",
            "no save commit was written and the branch did not move; the snapshot ref may have moved",
            "a repeat finishes the join once the failure above is fixed",
        )
    );
}

#[test]
fn missing_change_id_for_join_describes_the_possible_snapshot_move() {
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::ChangeIdMissing {
            head: gitio::Oid::parse("1111111111111111111111111111111111111111").unwrap(),
        }, None),
        expected_report(
            "join",
            "HEAD 1111111111111111111111111111111111111111 is this request's and zone's but carries no \
             Mortar-Change-Id",
            "no save commit was written and the branch did not move; the snapshot ref may have moved",
            "a repeat finishes the join once the failure above is fixed",
        )
    );
}

#[test]
fn bad_target_passes_through_to_snapshot_failure() {
    assert_eq!(
        snapshot_step_failure("resolve", &SnapshotError::BadTarget { reason: "attempt 0".to_string() }, None),
        crate::join::snapshot_failure("resolve", SnapshotError::BadTarget { reason: "attempt 0".to_string() })
    );
}

const MOVED: &str = "the snapshot ref moved as the work was saved first";

fn record() -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(), key: "key-1".to_string(), kind: "save".to_string(),
        actor: "a/r1".to_string(), reason: "save work".to_string(), at: 1,
        phase: OpPhase::Failed, moves: vec![],
        result: Some(OpResult::Stale { refname: "refs/snap/a".to_string(), actual: None }),
    }
}

#[test]
fn earlier_move_precedes_detail_for_step_failure() {
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::Git(git_error()), Some(MOVED)),
        Report {
            verb: "join".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(),
            owner: None, next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "git read-tree HEAD: exit code 128: fatal: index.lock exists".to_string(),
                "the branch did not move; the snapshot ref may have moved".to_string(),
                "a repeat finishes the join once the failure above is fixed".to_string(),
            ],
        }
    );
}

#[test]
fn branch_moved_gets_step_failure_and_repeat() {
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::BranchMoved(record()), None),
        expected_report(
            "join", "the branch refs/snap/a was not at the value read (it holds absent)",
            "the branch did not move; the snapshot ref may have moved",
            "a repeat finishes the join once the failure above is fixed",
        )
    );
}

#[test]
fn snapshot_ref_moved_fallthrough_reports_earlier_move() {
    assert_eq!(
        snapshot_step_failure("resolve", &SnapshotError::RefMoved(record()), Some(MOVED)),
        Report {
            verb: "resolve".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(),
            owner: None, next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "the snapshot ref refs/snap/a was not at the value read (it holds absent); this snapshot moved nothing".to_string(),
            ],
        }
    );
}

#[test]
fn repeated_operation_reports_earlier_move_before_attempt_detail() {
    assert_eq!(
        snapshot_step_failure("resolve", &SnapshotError::OpRepeated(record()), Some(MOVED)),
        Report {
            verb: "resolve".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(),
            owner: None, next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "the op log already holds a done record for key key-1 (op op-1); this attempt moved nothing".to_string(),
            ],
        }
    );
}

#[test]
fn index_not_set_names_where_the_branch_stands_and_the_earlier_move() {
    let commit = gitio::Oid::parse("2222222222222222222222222222222222222222").unwrap();
    assert_eq!(
        snapshot_step_failure("join", &SnapshotError::IndexNotSet { commit, error: git_error() }, Some(MOVED)),
        Report {
            verb: "join".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(),
            owner: None, next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "the folder's index could not be set to commit 2222222222222222222222222222222222222222's tree: git read-tree HEAD: exit code 128: fatal: index.lock exists; the index step left the files untouched, and the index is the old one, or that tree without its skip-worktree bits if the sparse-checkout reapply failed".to_string(),
                "the branch stands at commit 2222222222222222222222222222222222222222; the snapshot ref may have moved".to_string(),
                "a repeat finishes the join once the failure above is fixed".to_string(),
            ],
        }
    );
}
