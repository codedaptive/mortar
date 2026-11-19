//! snapshot_error_report: every SnapshotError arm under the verb "snapshot", in the cli's own words. An unborn or
//! detached HEAD and a path that is not UTF-8 are refusals (exit 1) with a next in words; a moved snapshot ref is busy
//! (exit 3); a repeated op is already done (exit 0); every other arm is a failure (exit 5) naming what happened. No
//! arm prints an inner error's Display (gitio's, oplog's, io's).

use std::path::PathBuf;

use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};
use snapshot::SnapshotError;

use super::snapshot_error_report;
use crate::report::{ExitCode, Report};

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const NEXT_HEAD: &str = "check out the request's branch in the folder, then repeat the same command";

fn oid() -> Oid {
    Oid::parse(HEAD).expect("a valid oid")
}

fn git_error() -> GitError {
    GitError {
        args: vec!["write-tree".to_string()],
        code: Some(128),
        stderr: "fatal: gitio text 5521".to_string(),
    }
}

fn record(result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "snapshot-0011223344556677".to_string(),
        kind: "snapshot".to_string(),
        actor: "harness".to_string(),
        reason: "end of turn".to_string(),
        at: 1_763_100_000,
        phase: OpPhase::Failed,
        moves: Vec::new(),
        result,
    }
}

fn report(exit: ExitCode, outcome: &str, lines: &[&str], owner: Option<&str>, next: &str) -> Report {
    Report {
        verb: "snapshot".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: owner.map(str::to_string),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(what: &str) -> Report {
    report(ExitCode::Failure, "failed", &[what], None, "repeat the same command")
}

#[test]
fn an_unborn_head_is_refused_naming_the_folder() {
    assert_eq!(
        snapshot_error_report(&SnapshotError::UnbornHead { folder: PathBuf::from("/w/a-r1") }),
        report(
            ExitCode::Refused,
            "refused",
            &["HEAD of the folder /w/a-r1 has no commit", "nothing was snapshotted"],
            Some("the folder's HEAD"),
            NEXT_HEAD,
        )
    );
}

#[test]
fn a_detached_head_is_refused_naming_the_folder() {
    assert_eq!(
        snapshot_error_report(&SnapshotError::DetachedHead { folder: PathBuf::from("/w/a-r1") }),
        report(
            ExitCode::Refused,
            "refused",
            &["HEAD of the folder /w/a-r1 names no branch", "nothing was snapshotted"],
            Some("the folder's HEAD"),
            NEXT_HEAD,
        )
    );
}

#[test]
fn a_path_that_is_not_utf8_is_refused_naming_the_file() {
    assert_eq!(
        snapshot_error_report(&SnapshotError::PathNotUtf8 { path: PathBuf::from("data/big.bin") }),
        report(
            ExitCode::Refused,
            "refused",
            &["the refused file \"data/big.bin\" has a name that is not UTF-8, so it cannot be passed to git; nothing \
               was snapshotted"],
            Some("the file data/big.bin"),
            "rename the file to a UTF-8 name, then repeat the same command",
        )
    );
}

#[test]
fn a_moved_snapshot_ref_is_busy_and_nothing_moved() {
    let stale = OpResult::Stale { refname: "refs/mortar/snap/core/fix-1".to_string(), actual: Some(oid()) };
    assert_eq!(
        snapshot_error_report(&SnapshotError::RefMoved(record(Some(stale)))),
        report(
            ExitCode::Busy,
            "busy",
            &["the snapshot ref refs/mortar/snap/core/fix-1 moved while snapshot ran; nothing moved"],
            None,
            "repeat the same command later",
        )
    );
}

#[test]
fn a_moved_snapshot_ref_without_a_stale_result_is_busy() {
    assert_eq!(
        snapshot_error_report(&SnapshotError::RefMoved(record(None))),
        report(
            ExitCode::Busy,
            "busy",
            &["the snapshot ref moved while snapshot ran; nothing moved"],
            None,
            "repeat the same command later",
        )
    );
}

#[test]
fn a_repeated_op_is_already_done() {
    assert_eq!(
        snapshot_error_report(&SnapshotError::OpRepeated(record(Some(OpResult::Moved)))),
        report(ExitCode::Done, "already-done", &["this snapshot was already done"], None, "nothing to do")
    );
}

#[test]
fn a_bad_target_is_a_failure_with_its_reason() {
    let error = SnapshotError::BadTarget { reason: "the zone \"..\" cannot name a ref".to_string() };
    assert_eq!(
        snapshot_error_report(&error),
        failed("the snapshot ref cannot be named: the zone \"..\" cannot name a ref")
    );
}

#[test]
fn git_is_a_failure_in_the_clis_words() {
    assert_eq!(
        snapshot_error_report(&SnapshotError::Git(git_error())),
        failed("git failed while snapshotting the folder")
    );
}

#[test]
fn oplog_is_a_failure_in_the_clis_words() {
    let error = SnapshotError::Oplog(OplogError::Malformed { line: None, reason: "no op_id".to_string() });
    assert_eq!(snapshot_error_report(&error), failed("the op log could not be read or written"));
}

#[test]
fn a_file_size_error_is_a_failure_naming_the_file() {
    let error = SnapshotError::FileSize {
        path: PathBuf::from("data/loop"),
        error: std::io::Error::other("too many levels of symbolic links"),
    };
    assert_eq!(snapshot_error_report(&error), failed("the size of data/loop could not be read"));
}

#[test]
fn a_cap_mismatch_is_a_failure_naming_the_paths() {
    let error = SnapshotError::CapMismatch { paths: vec![PathBuf::from("a/x.bin"), PathBuf::from("b/y.log")] };
    assert_eq!(
        snapshot_error_report(&error),
        failed(
            "the capped tree did not differ from the full tree by the refused files only: a/x.bin, b/y.log; \
             GIT_NOGLOB_PATHSPECS or GIT_LITERAL_PATHSPECS may be set, or the folder changed while snapshot ran"
        )
    );
}

#[test]
fn an_index_not_set_is_a_failure_naming_the_commit() {
    let error = SnapshotError::IndexNotSet { commit: oid(), error: git_error() };
    assert_eq!(
        snapshot_error_report(&error),
        failed("commit 0123456789abcdef0123456789abcdef01234567 was written but the folder's index was not set")
    );
}

#[test]
fn a_trailers_error_is_a_failure_in_the_clis_words() {
    let error = SnapshotError::Trailers(MortarTrailerError::Git(git_error()));
    assert_eq!(snapshot_error_report(&error), failed("HEAD's Mortar trailers could not be read"));
}

#[test]
fn a_change_id_error_is_a_failure_in_the_clis_words() {
    let error =
        SnapshotError::ChangeId(ChangeIdError::Malformed { text: "abc".to_string(), reason: "too short".to_string() });
    assert_eq!(snapshot_error_report(&error), failed("a new change ID could not be drawn"));
}

#[test]
fn a_bad_sentence_is_a_failure_naming_its_reason() {
    let error = SnapshotError::BadSentence { reason: "the sentence is empty".to_string() };
    assert_eq!(snapshot_error_report(&error), failed("the sentence was refused: the sentence is empty"));
}

#[test]
fn a_head_without_a_change_id_is_a_failure_naming_the_commit() {
    assert_eq!(
        snapshot_error_report(&SnapshotError::ChangeIdMissing { head: oid() }),
        failed("HEAD 0123456789abcdef0123456789abcdef01234567 carries no Mortar-Change-Id")
    );
}

#[test]
fn a_moved_branch_is_a_failure_naming_what_happened() {
    let stale = OpResult::Stale { refname: "refs/heads/a/r1".to_string(), actual: Some(oid()) };
    assert_eq!(
        snapshot_error_report(&SnapshotError::BranchMoved(record(Some(stale)))),
        failed("the folder's branch moved while snapshot ran")
    );
}

#[test]
fn no_arm_prints_an_inner_errors_display() {
    let errors = [
        SnapshotError::Git(git_error()),
        SnapshotError::IndexNotSet { commit: oid(), error: git_error() },
        SnapshotError::Trailers(MortarTrailerError::Git(git_error())),
        SnapshotError::Oplog(OplogError::Malformed { line: None, reason: "no op_id".to_string() }),
        SnapshotError::ChangeId(ChangeIdError::Malformed { text: "abc".to_string(), reason: "too short".to_string() }),
        SnapshotError::FileSize { path: PathBuf::from("data/loop"), error: std::io::Error::other("io text 7731") },
    ];
    for error in errors {
        let printed = format!("{:?}", snapshot_error_report(&error));
        for inner in ["5521", "write-tree", "malformed op data", "too short", "7731"] {
            assert!(!printed.contains(inner), "{inner} in {printed}");
        }
    }
}

#[test]
fn every_report_is_under_the_verb_snapshot() {
    let errors = [
        SnapshotError::UnbornHead { folder: PathBuf::from("/w") },
        SnapshotError::RefMoved(record(None)),
        SnapshotError::OpRepeated(record(None)),
        SnapshotError::Git(git_error()),
    ];
    for error in errors {
        assert_eq!(snapshot_error_report(&error).verb, "snapshot");
    }
}
