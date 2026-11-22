//! saved_state_report: the cli's words for saved_state's Git, FileSize and BadTarget errors with none of the inner
//! error's text, and UnbornHead as the HEAD refusal under the given verb: next a repeat, or for an attempt folder
//! checking out its try branch.
//! Unit tests: the errors are built as data; report::failure_report, report::head_refusal and
//! report::repeat_head_refusal are pure.

use std::path::PathBuf;

use gitio::GitError;
use snapshot::SnapshotError;

use super::saved_state_report;
use crate::report::{ExitCode, Report};

fn failed(verb: &str, what: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![what.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

fn assert_absent(report: &Report, marker: &str) {
    assert!(report.lines.iter().all(|line| !line.contains(marker)), "{marker:?} shows in {:?}", report.lines);
}

#[test]
fn git_is_the_clis_tree_line_under_the_verb_without_gitios_text() {
    let error = SnapshotError::Git(GitError {
        args: vec!["write-tree".to_string()],
        code: Some(128),
        stderr: "fatal: GITMARKER-tree-broke\n".to_string(),
    });
    let report = saved_state_report(&error, "r1", "z1", "status", None);
    assert_eq!(report, failed("status", "git failed while building the folder's tree"));
    assert_absent(&report, "GITMARKER");
    assert_absent(&report, "write-tree");
}

#[test]
fn file_size_names_the_path_without_the_io_errors_text() {
    let error = SnapshotError::FileSize {
        path: PathBuf::from("src/loop.rs"),
        error: std::io::Error::other("IOMARKER too many links"),
    };
    let report = saved_state_report(&error, "r1", "z1", "disk", None);
    assert_eq!(report, failed("disk", "the size of src/loop.rs could not be read"));
    assert_absent(&report, "IOMARKER");
}

#[test]
fn bad_target_names_request_and_zone_without_its_reason() {
    let error = SnapshotError::BadTarget { reason: "REASONMARKER attempt 0 is not 1 or more".to_string() };
    let report = saved_state_report(&error, "fix-parser", "core", "status", None);
    assert_eq!(report, failed("status", "the snapshot ref of request fix-parser in zone core cannot be named"));
    assert_absent(&report, "REASONMARKER");
}

#[test]
fn unborn_head_is_the_head_refusal_whose_next_is_a_repeat_under_the_given_verb() {
    let error = SnapshotError::UnbornHead { folder: PathBuf::from("/work/req") };
    let report = saved_state_report(&error, "r1", "z1", "disk", None);
    assert_eq!(
        report,
        Report {
            verb: "disk".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["HEAD of the folder /work/req has no commit".to_string()],
            owner: Some("the folder's HEAD".to_string()),
            next: "check out the request's branch in the folder, then repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn unborn_head_in_an_attempt_folder_names_the_try_branch_as_its_next() {
    let error = SnapshotError::UnbornHead { folder: PathBuf::from("/work/try-1") };
    let report = saved_state_report(&error, "task", "core", "status", Some("try/core/task/1"));
    assert_eq!(
        report,
        Report {
            verb: "status".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["HEAD of the folder /work/try-1 has no commit".to_string()],
            owner: Some("the folder's HEAD".to_string()),
            next: "check out the branch try/core/task/1 in the attempt's folder, then repeat the same command"
                .to_string(),
            fields: Vec::new(),
        }
    );
}
