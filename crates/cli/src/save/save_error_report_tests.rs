//! save_error_report: each SnapshotError's exit, outcome, lines, owner and next, whether the lines are cli's
//! own text, a reason carried as data, or gitio's or oplog's own text (an index not set, git, op log, trailers and
//! change ID errors).
//! Unit tests: report::failure_report is pure and used directly; the errors are built as data, and the
//! gitio and oplog lines come from their exported, pure Display impls.

use std::path::PathBuf;

use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};
use snapshot::SnapshotError;

use super::save_error_report;
use crate::report::{ExitCode, Report};

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

fn oid() -> Oid {
    Oid::parse(HEAD).unwrap()
}

fn git_error() -> GitError {
    GitError {
        args: vec!["read-tree".to_string(), "HEAD".to_string()],
        code: Some(128),
        stderr: "fatal: index.lock exists\n".to_string(),
    }
}

fn record(result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "save-0011223344556677".to_string(),
        kind: "save".to_string(),
        actor: "a/r1".to_string(),
        reason: "fix the parser".to_string(),
        at: 1_763_100_000,
        phase: OpPhase::Failed,
        moves: Vec::new(),
        result,
    }
}

fn report(exit: ExitCode, outcome: &str, lines: &[&str], owner: Option<&str>, next: &str) -> Report {
    Report {
        verb: "save".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: owner.map(str::to_string),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(what: &str) -> Report {
    report(ExitCode::Failure, "failed", &[what], None, "repeat the same command")
}

#[test]
fn a_bad_sentence_is_refused_with_its_reason() {
    let error = SnapshotError::BadSentence { reason: "the sentence is empty or only whitespace".to_string() };
    assert_eq!(
        save_error_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &["the sentence is empty or only whitespace", "nothing ran"],
            None,
            "mr save \"<one line saying what changed>\"",
        )
    );
}

#[test]
fn an_unborn_head_is_refused_naming_the_folder() {
    let error = SnapshotError::UnbornHead { folder: PathBuf::from("/w/a-r1") };
    assert_eq!(
        save_error_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &["HEAD of the folder /w/a-r1 has no commit", "no save commit was written"],
            Some("the folder's HEAD"),
            "check out the request's branch in this folder, then mr save again",
        )
    );
}

#[test]
fn a_detached_head_is_refused_naming_the_folder() {
    let error = SnapshotError::DetachedHead { folder: PathBuf::from("/w/a-r1") };
    assert_eq!(
        save_error_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &["HEAD of the folder /w/a-r1 names no branch", "no save commit was written"],
            Some("the folder's HEAD"),
            "check out the request's branch in this folder, then mr save again",
        )
    );
}

#[test]
fn a_head_without_a_change_id_is_refused_naming_the_commit() {
    assert_eq!(
        save_error_report(&SnapshotError::ChangeIdMissing { head: oid() }),
        report(
            ExitCode::Refused,
            "refused",
            &[
                "HEAD 0123456789abcdef0123456789abcdef01234567 is this request's but carries no Mortar-Change-Id",
                "no save commit was written; the snapshot ref may have moved",
            ],
            Some("the commit 0123456789abcdef0123456789abcdef01234567"),
            "ask the zone's coordinator to mend HEAD's trailers",
        )
    );
}

#[test]
fn a_path_that_is_not_utf8_is_refused_naming_the_file() {
    assert_eq!(
        save_error_report(&SnapshotError::PathNotUtf8 { path: PathBuf::from("data/big.bin") }),
        report(
            ExitCode::Refused,
            "refused",
            &["the refused file \"data/big.bin\" has a name that is not UTF-8, so it cannot be passed to git; this snapshot \
               wrote no commit"],
            Some("the file data/big.bin"),
            "rename the file to a UTF-8 name, then mr save again",
        )
    );
}

#[test]
fn a_moved_branch_is_busy_naming_the_ref() {
    let stale = OpResult::Stale { refname: "refs/heads/a/r1".to_string(), actual: Some(oid()) };
    assert_eq!(
        save_error_report(&SnapshotError::BranchMoved(record(Some(stale)))),
        report(
            ExitCode::Busy,
            "busy",
            &[
                "the branch refs/heads/a/r1 moved while save ran",
                "this save did not move the branch; the snapshot ref may have moved",
            ],
            None,
            "repeat the same command later",
        )
    );
}

#[test]
fn a_moved_branch_without_a_stale_result_is_busy() {
    assert_eq!(
        save_error_report(&SnapshotError::BranchMoved(record(None))),
        report(
            ExitCode::Busy,
            "busy",
            &[
                "the branch moved while save ran",
                "this save did not move the branch; the snapshot ref may have moved",
            ],
            None,
            "repeat the same command later",
        )
    );
}

#[test]
fn a_moved_snapshot_ref_without_a_stale_result_is_busy() {
    assert_eq!(
        save_error_report(&SnapshotError::RefMoved(record(None))),
        report(
            ExitCode::Busy,
            "busy",
            &["the snapshot ref moved while save ran; this snapshot moved nothing"],
            None,
            "repeat the same command later",
        )
    );
}

#[test]
fn a_repeated_op_is_already_done() {
    assert_eq!(
        save_error_report(&SnapshotError::OpRepeated(record(Some(OpResult::Moved)))),
        report(ExitCode::Done, "already-done", &["this save was already done"], None, "mr join")
    );
}

#[test]
fn a_cap_mismatch_is_a_failure_naming_the_paths() {
    let error = SnapshotError::CapMismatch { paths: vec![PathBuf::from("a/x.bin"), PathBuf::from("b/[y].log")] };
    assert_eq!(
        save_error_report(&error),
        failed(
            "the capped tree did not differ from the full tree by the refused files only: a/x.bin, b/[y].log; \
             GIT_NOGLOB_PATHSPECS or GIT_LITERAL_PATHSPECS may be set, or the folder changed while save ran"
        )
    );
}

#[test]
fn a_file_size_error_is_a_failure_naming_the_file() {
    let error = SnapshotError::FileSize {
        path: PathBuf::from("data/loop"),
        error: std::io::Error::other("too many levels of symbolic links"),
    };
    assert_eq!(
        save_error_report(&error),
        failed("the size of data/loop could not be read: too many levels of symbolic links")
    );
}

#[test]
fn a_bad_target_is_a_failure_with_its_reason() {
    let error = SnapshotError::BadTarget { reason: "the zone \"..\" cannot name a ref".to_string() };
    assert_eq!(save_error_report(&error), failed("the snapshot ref cannot be named: the zone \"..\" cannot name a ref"));
}

#[test]
fn an_index_not_set_is_a_failure_a_repeat_mends() {
    assert_eq!(
        save_error_report(&SnapshotError::IndexNotSet { commit: oid(), error: git_error() }),
        report(
            ExitCode::Failure,
            "failed",
            &[
                "the folder's index could not be set to commit 0123456789abcdef0123456789abcdef01234567's tree: git \
                 read-tree HEAD: exit code 128: fatal: index.lock exists",
                "the branch stands at commit 0123456789abcdef0123456789abcdef01234567; the snapshot ref may have moved",
                "the index step left the files untouched; the index is the old one, or the saved tree without its \
                 skip-worktree bits if the sparse-checkout reapply failed; a repeat sets it",
            ],
            None,
            "repeat the same command",
        )
    );
}

#[test]
fn a_git_error_is_a_failure_with_its_text() {
    assert_eq!(
        save_error_report(&SnapshotError::Git(git_error())),
        failed("git read-tree HEAD: exit code 128: fatal: index.lock exists")
    );
}

#[test]
fn an_oplog_error_is_a_failure_with_its_text() {
    let error = SnapshotError::Oplog(OplogError::Malformed { line: None, reason: "no op_id".to_string() });
    assert_eq!(
        save_error_report(&error),
        failed("malformed op data: no op_id; mortar did not write this text, so check what did")
    );
}

#[test]
fn a_trailers_error_is_a_failure_with_its_text() {
    let error = SnapshotError::Trailers(MortarTrailerError::Git(git_error()));
    assert_eq!(
        save_error_report(&error),
        failed("git trailers failed: git read-tree HEAD: exit code 128: fatal: index.lock exists")
    );
}

#[test]
fn a_change_id_error_is_a_failure_with_its_text() {
    let error = SnapshotError::ChangeId(ChangeIdError::Malformed { text: "abc".to_string(), reason: "too short".to_string() });
    assert_eq!(save_error_report(&error), failed("not a change ID: \"abc\": too short"));
}
