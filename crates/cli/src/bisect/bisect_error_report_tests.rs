//! bisect_error_report: every BisectError arm, CulpritError's two included, in the cli's own words; never gitio's,
//! oplog's or a probe's OS text.

use gitio::{GitError, Oid};
use land::{BisectError, CulpritError};
use oplog::MortarTrailerError;

use super::bisect_error_report;
use crate::report::{ExitCode, Report};

const LAST: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const COMMIT: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const FAILED: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refused(line: &str, owner: &str, next: &str) -> Report {
    Report {
        verb: "bisect".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line.to_string()],
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(line: &str) -> Report {
    Report {
        verb: "bisect".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string(), "nothing half-done is left".to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn a_missing_origin_is_refused_naming_the_ref() {
    let error = BisectError::OriginMissing { refname: "refs/heads/main".to_string() };
    assert_eq!(
        bisect_error_report(&error),
        refused(
            "refs/heads/main does not exist, so bisect has no origin line to search",
            "the repository's branches",
            "create refs/heads/main, then repeat the same command",
        )
    );
}

#[test]
fn a_failed_commit_off_origin_is_refused_naming_it_and_the_branch() {
    let error = BisectError::NotOnOrigin { failed: oid(FAILED), origin_branch: "main".to_string() };
    assert_eq!(
        bisect_error_report(&error),
        refused(
            "the failed commit e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 is not on main's first-parent line, so it \
             names no landing",
            "the commits named",
            "name a failed commit on main's first-parent line: mortar bisect <last pass> <failed> --timeout \
             <seconds> --actor <name> -- <command>...",
        )
    );
}

#[test]
fn a_last_pass_off_the_failed_line_is_refused_naming_both() {
    let error = BisectError::OffTheLine { last_pass: oid(LAST), failed: oid(FAILED) };
    assert_eq!(
        bisect_error_report(&error),
        refused(
            "the last passing commit 4b825dc642cb6eb9a060e54bf8d69288fbee4904 is not on \
             e69de29bb2d1d6434b8b29ae775ad8c2e48c5391's first-parent line, so the range names no landings",
            "the commits named",
            "name a last passing commit on e69de29bb2d1d6434b8b29ae775ad8c2e48c5391's first-parent line: mortar \
             bisect <last pass> e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 --timeout <seconds> --actor <name> -- \
             <command>...",
        )
    );
}

#[test]
fn a_probe_that_could_not_run_is_refused_naming_the_commit() {
    let error = BisectError::ProbeCouldNotRun {
        commit: oid(COMMIT),
        error: "No such file or directory (os error 2)".to_string(),
    };
    assert_eq!(
        bisect_error_report(&error),
        refused(
            "the test command could not be run at 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e, so its verdict there \
             is unknown and the search stopped",
            "the test command",
            "check that the test command's program exists and can be run, then repeat the same command",
        )
    );
}

#[test]
fn a_probe_that_could_not_run_never_prints_its_error_text() {
    let error = BisectError::ProbeCouldNotRun { commit: oid(COMMIT), error: "spawn failed: EACCES-7731".to_string() };
    let report = format!("{:?}", bisect_error_report(&error));
    assert!(!report.contains("EACCES-7731"), "{report}");
    assert!(!report.contains("spawn failed"), "{report}");
}

#[test]
fn git_is_a_failure_in_the_clis_words() {
    let error = BisectError::Git(GitError::refusal(vec!["rev-list".to_string()], "fatal: gitio text"));
    assert_eq!(bisect_error_report(&error), failed("git failed while reading origin's line or probing a commit"));
}

#[test]
fn a_first_failing_commit_that_is_not_a_landing_is_refused_naming_its_parent_count() {
    let error = BisectError::Culprit(CulpritError::NotALanding { commit: oid(COMMIT), parents: 3 });
    assert_eq!(
        bisect_error_report(&error),
        refused(
            "the first failing commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e has 3 parents, so it is not a \
             landing (a landing has one or two)",
            "the commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e",
            "inspect commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e by hand: Mortar did not land it, so it names \
             no squash",
        )
    );
    let root = BisectError::Culprit(CulpritError::NotALanding { commit: oid(COMMIT), parents: 0 });
    assert_eq!(
        bisect_error_report(&root).lines,
        vec![
            "the first failing commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e has 0 parents, so it is not a landing \
             (a landing has one or two)"
                .to_string()
        ]
    );
}

#[test]
fn unreadable_culprit_trailers_are_a_failure_in_the_clis_words() {
    let error = BisectError::Culprit(CulpritError::Trailers(MortarTrailerError::Git(GitError::refusal(
        Vec::new(),
        "trailer text",
    ))));
    assert_eq!(bisect_error_report(&error), failed("the first failing landing's squash trailers could not be read"));
}
