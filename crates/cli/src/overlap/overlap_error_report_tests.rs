//! overlap_error_report: every JoinError arm, matched one by one, as a failure (exit 5) of the read-only overlap verb
//! in the cli's own words; gitio's Display text is never printed.

use ::join::JoinError;
use gitio::{GitError, Oid};

use super::overlap_error_report;
use crate::report::{ExitCode, Report};

const COMMIT: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";

fn oid() -> Oid {
    Oid::parse(COMMIT).expect("a valid oid")
}

fn git_error() -> GitError {
    GitError {
        args: vec!["diff".to_string(), "--name-only".to_string()],
        code: Some(128),
        stderr: "fatal: gitio text 5521".to_string(),
    }
}

fn failed(what: &str) -> Report {
    Report {
        verb: "overlap".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![what.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn git_is_a_failure_in_the_clis_words() {
    assert_eq!(
        overlap_error_report(&JoinError::Git(git_error())),
        failed("git failed while reading the requests' changed files")
    );
}

#[test]
fn git_never_prints_gitios_text() {
    let printed = format!("{:?}", overlap_error_report(&JoinError::Git(git_error())));
    assert!(!printed.contains("5521"), "{printed}");
    assert!(!printed.contains("--name-only"), "{printed}");
}

#[test]
fn a_missing_onto_commit_names_it() {
    assert_eq!(
        overlap_error_report(&JoinError::OntoMissing(oid())),
        failed("commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e does not exist here")
    );
}

#[test]
fn a_missing_tip_names_the_ref() {
    assert_eq!(
        overlap_error_report(&JoinError::TipMissing("refs/heads/req/core/fix-1".to_string())),
        failed("refs/heads/req/core/fix-1 does not exist")
    );
}

#[test]
fn a_merge_commit_names_it() {
    assert_eq!(
        overlap_error_report(&JoinError::MergeCommit(oid())),
        failed("commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e is a merge")
    );
}

#[test]
fn a_root_commit_names_it() {
    assert_eq!(
        overlap_error_report(&JoinError::RootCommit(oid())),
        failed("commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e has no parent")
    );
}

#[test]
fn an_empty_author_date_names_the_commit() {
    assert_eq!(
        overlap_error_report(&JoinError::EmptyAuthorDate(oid())),
        failed("commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e has an empty author date")
    );
}

#[test]
fn a_tip_not_replayed_names_the_ref() {
    assert_eq!(
        overlap_error_report(&JoinError::TipNotReplayed("refs/heads/req/core/fix-1".to_string())),
        failed("git gave no update for refs/heads/req/core/fix-1")
    );
}
