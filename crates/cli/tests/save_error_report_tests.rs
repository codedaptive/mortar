//! save_error_report: the SnapshotErrors whose lines carry gitio's or oplog's own text (an index not set, git, op log,
//! trailers and change ID errors): exit, outcome, lines, owner and next (S-71). The errors whose lines are cli's own
//! text are the ZUTs in src/save/save_error_report_tests.rs.
//! ZIT (S-79 as replaced: the line is the file): it calls report::failure_report for real.

use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OplogError};
use snapshot::SnapshotError;

use cli::{save_error_report, ExitCode, Report};

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
fn an_index_not_set_is_a_failure_a_repeat_mends() {
    assert_eq!(
        save_error_report(&SnapshotError::IndexNotSet { commit: oid(), error: git_error() }),
        report(
            ExitCode::Failure,
            "failed",
            &[
                "saved commit 0123456789abcdef0123456789abcdef01234567 but the folder's index was not set: git \
                 read-tree HEAD: exit code 128: fatal: index.lock exists",
                "the files are untouched and the index is the old one, or the saved tree without its skip-worktree \
                 bits if the sparse-checkout reapply failed; a repeat sets it (S-41, S-125)",
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
