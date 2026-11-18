//! save_error_report: each SnapshotError's exit, outcome, lines, owner and next (S-71) where the lines are cli's own
//! text or a reason carried as data. The errors whose lines are gitio's or oplog's own text (an index not set, git,
//! op log, trailers and change ID errors) are ZITs in crates/cli/tests/save_error_report_tests.rs.
//! ZUT (rule 11): report::failure_report is pure and used directly; the errors are built as data.

use std::path::PathBuf;

use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult};
use snapshot::SnapshotError;

use super::save_error_report;
use crate::report::{ExitCode, Report};

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";

fn oid() -> Oid {
    Oid::parse(HEAD).unwrap()
}

fn record(result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "save-0011223344556677".to_string(),
        kind: "save".to_string(),
        actor: "orc/r1".to_string(),
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
    report(ExitCode::Failure, "failed", &[what, "nothing half-done is left"], None, "repeat the same command")
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
    let error = SnapshotError::UnbornHead { folder: PathBuf::from("/w/orc-r1") };
    assert_eq!(
        save_error_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &["HEAD of the folder /w/orc-r1 has no commit", "nothing was saved"],
            Some("the folder's HEAD"),
            "check out the request's branch in this folder, then mr save again",
        )
    );
}

#[test]
fn a_detached_head_is_refused_naming_the_folder() {
    let error = SnapshotError::DetachedHead { folder: PathBuf::from("/w/orc-r1") };
    assert_eq!(
        save_error_report(&error),
        report(
            ExitCode::Refused,
            "refused",
            &["HEAD of the folder /w/orc-r1 names no branch", "nothing was saved"],
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
                "HEAD 0123456789abcdef0123456789abcdef01234567 is this request's but carries no Mortar-Change-Id \
                 (S-34)",
                "no commit was written; the snapshot ref may have moved",
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
            &["the refused file \"data/big.bin\" has a name that is not UTF-8, so it cannot be passed to git; no \
               commit was written"],
            Some("the file data/big.bin"),
            "rename the file to a UTF-8 name, then mr save again",
        )
    );
}

#[test]
fn a_moved_branch_is_busy_naming_the_ref() {
    let stale = OpResult::Stale { refname: "refs/heads/orc/r1".to_string(), actual: Some(oid()) };
    assert_eq!(
        save_error_report(&SnapshotError::BranchMoved(record(Some(stale)))),
        report(
            ExitCode::Busy,
            "busy",
            &["the branch refs/heads/orc/r1 moved while save ran; this save did not move it, and the snapshot ref \
               may have moved"],
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
            &["the snapshot ref moved while save ran; nothing moved"],
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
