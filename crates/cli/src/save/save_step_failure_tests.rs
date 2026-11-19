//! save_step_failure: save's own lines for its Git, Trailers, ChangeId and Oplog errors, and every other error
//! passed to save_error_report. ZUT: the errors are built as data.

use gitio::GitError;
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};
use snapshot::SnapshotError;

use super::{save_error_report, save_step_failure};
use crate::report::{ExitCode, Report};

fn git_error() -> GitError {
    GitError {
        args: vec!["read-tree".to_string(), "HEAD".to_string()],
        code: Some(128),
        stderr: "fatal: index.lock exists\n".to_string(),
    }
}

#[test]
fn git_failure_describes_the_possible_snapshot_move() {
    assert_eq!(
        save_step_failure(&SnapshotError::Git(git_error()), None),
        Report {
            verb: "save".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(), owner: None,
            next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "git read-tree HEAD: exit code 128: fatal: index.lock exists".to_string(),
                "the branch did not move; the snapshot ref may have moved".to_string(),
                "a repeat finishes the save once the failure above is fixed".to_string(),
            ],
        }
    );
}

#[test]
fn trailers_failure_describes_the_possible_snapshot_move() {
    assert_eq!(
        save_step_failure(&SnapshotError::Trailers(MortarTrailerError::Git(git_error())), None),
        Report {
            verb: "save".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(), owner: None,
            next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "git trailers failed: git read-tree HEAD: exit code 128: fatal: index.lock exists".to_string(),
                "no save commit was written and the branch did not move; the snapshot ref may have moved".to_string(),
                "a repeat finishes the save once the failure above is fixed".to_string(),
            ],
        }
    );
}

#[test]
fn change_id_failure_describes_the_possible_snapshot_move() {
    assert_eq!(
        save_step_failure(&SnapshotError::ChangeId(ChangeIdError::Malformed {
            text: "abc".to_string(), reason: "too short".to_string(),
        }), None),
        Report {
            verb: "save".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(), owner: None,
            next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "not a change ID: \"abc\": too short".to_string(),
                "no save commit was written and the branch did not move; the snapshot ref may have moved".to_string(),
                "a repeat finishes the save once the failure above is fixed".to_string(),
            ],
        }
    );
}

#[test]
fn oplog_failure_before_the_intent_says_an_op_may_be_open() {
    assert_eq!(
        save_step_failure(&SnapshotError::Oplog(OplogError::Malformed {
            line: None, reason: "no op_id".to_string(),
        }), None),
        Report {
            verb: "save".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(), owner: None,
            next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "malformed op data: no op_id; mortar did not write this text, so check what did".to_string(),
                "the snapshot ref and the branch may have moved, and an op may be left open; the next command that takes the join lock settles it".to_string(),
                "a repeat finishes the save once the failure above is fixed".to_string(),
            ],
        }
    );
}

#[test]
fn oplog_failure_after_the_intent_says_an_op_may_be_open() {
    let transaction = GitError {
        args: vec!["update-ref".to_string(), "--stdin".to_string()],
        code: Some(128),
        stderr: "fatal: cannot lock ref 'refs/heads/main'\n".to_string(),
    };
    assert_eq!(
        save_step_failure(&SnapshotError::Oplog(OplogError::Git(transaction)), None),
        Report {
            verb: "save".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(), owner: None,
            next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "git update-ref --stdin: exit code 128: fatal: cannot lock ref 'refs/heads/main'; fix what git reports, then run again".to_string(),
                "the snapshot ref and the branch may have moved, and an op may be left open; the next command that takes the join lock settles it".to_string(),
                "a repeat finishes the save once the failure above is fixed".to_string(),
            ],
        }
    );
}

#[test]
fn other_errors_pass_through_to_save_error_report() {
    let error = SnapshotError::BadSentence { reason: "the sentence is empty or only whitespace".to_string() };
    assert_eq!(save_step_failure(&error, None), save_error_report(&error));
}

const MOVED: &str = "the snapshot ref moved as the work was saved first";

#[test]
fn refusal_puts_earlier_snapshot_move_first() {
    let head = gitio::Oid::parse("1111111111111111111111111111111111111111").unwrap();
    assert_eq!(
        save_step_failure(&SnapshotError::ChangeIdMissing { head }, Some(MOVED)),
        Report {
            verb: "save".to_string(), exit: ExitCode::Refused, outcome: "refused".to_string(),
            owner: Some("the commit 1111111111111111111111111111111111111111".to_string()),
            next: "ask the zone's coordinator to mend HEAD's trailers".to_string(), fields: vec![],
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "HEAD 1111111111111111111111111111111111111111 is this request's but carries no Mortar-Change-Id".to_string(),
                "no save commit was written; the snapshot ref may have moved".to_string(),
            ],
        }
    );
}

#[test]
fn busy_snapshot_ref_puts_earlier_move_first() {
    let record = OpRecord {
        op_id: "op-1".to_string(), key: "key-1".to_string(), kind: "save".to_string(),
        actor: "a/r1".to_string(), reason: "save work".to_string(), at: 1,
        phase: OpPhase::Failed, moves: vec![], result: Some(OpResult::Moved),
    };
    assert_eq!(
        save_step_failure(&SnapshotError::RefMoved(record), Some(MOVED)),
        Report {
            verb: "save".to_string(), exit: ExitCode::Busy, outcome: "busy".to_string(),
            owner: None, next: "repeat the same command later".to_string(), fields: vec![],
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "the snapshot ref moved while save ran; this snapshot moved nothing".to_string(),
            ],
        }
    );
}

#[test]
fn step_failure_places_earlier_move_before_detail() {
    assert_eq!(
        save_step_failure(&SnapshotError::Git(git_error()), Some(MOVED)),
        Report {
            verb: "save".to_string(), exit: ExitCode::Failure, outcome: "failed".to_string(),
            owner: None, next: "repeat the same command".to_string(), fields: vec![],
            lines: vec![
                "the snapshot ref moved as the work was saved first".to_string(),
                "git read-tree HEAD: exit code 128: fatal: index.lock exists".to_string(),
                "the branch did not move; the snapshot ref may have moved".to_string(),
                "a repeat finishes the save once the failure above is fixed".to_string(),
            ],
        }
    );
}
