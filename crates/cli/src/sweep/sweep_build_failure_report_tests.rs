//! sweep_build_failure_report: sweep_refs deleted its refs, then sweep_build failed. Exit 5, naming each deleted ref
//! and why the build dirs were not cleared; it never says nothing half-done is left (the refs are gone), and a repeat
//! sweeps only the build dirs.

use std::path::PathBuf;

use folders::FolderError;
use gitio::GitError;
use oplog::OplogError;

use super::sweep_build_failure_report;
use crate::report::{ExitCode, OutValue, Report};

const FIRST: &str = "the refs listed were deleted; clearing the idle build dirs failed";

fn deleted() -> Vec<String> {
    vec!["refs/mortar/replayed/gone/abc".to_string(), "refs/t3/old".to_string()]
}

fn expected(why: &str) -> Report {
    Report {
        verb: "sweep".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![
            FIRST.to_string(),
            "deleted refs/mortar/replayed/gone/abc".to_string(),
            "deleted refs/t3/old".to_string(),
            why.to_string(),
        ],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: vec![(
            "deleted".to_string(),
            OutValue::Texts(vec!["refs/mortar/replayed/gone/abc".to_string(), "refs/t3/old".to_string()]),
        )],
    }
}

fn git_error() -> GitError {
    GitError { args: vec!["rev-parse".to_string()], code: Some(128), stderr: "fatal: gitio text 5521".to_string() }
}

#[test]
fn an_io_failure_lists_the_deleted_refs_then_folder_error_whys_line() {
    let error = FolderError::Io { path: PathBuf::from("/w/fix-1/target"), source: std::io::Error::other("disk gone") };
    assert_eq!(sweep_build_failure_report(&deleted(), &error), expected("/w/fix-1/target: disk gone"));
}

#[test]
fn a_git_failure_is_in_the_clis_words_without_gitios_text() {
    let report = sweep_build_failure_report(&deleted(), &FolderError::Git(git_error()));
    assert_eq!(report, expected("a git run failed"));
    assert!(!format!("{report:?}").contains("5521"), "{report:?}");
}

#[test]
fn an_oplog_failure_is_in_the_clis_words() {
    let error = FolderError::Oplog(OplogError::Malformed { line: None, reason: "no op_id".to_string() });
    assert_eq!(sweep_build_failure_report(&deleted(), &error), expected("the op log could not be read or written"));
}

#[test]
fn a_disk_probe_failure_names_the_path_and_reason() {
    let error = FolderError::DiskProbe { path: PathBuf::from("/w"), reason: "df failed".to_string() };
    assert_eq!(
        sweep_build_failure_report(&deleted(), &error),
        expected("the free disk at /w could not be read: df failed")
    );
}

#[test]
fn it_never_says_nothing_half_done_is_left() {
    let errors = [
        FolderError::Git(git_error()),
        FolderError::Io { path: PathBuf::from("/w/a"), source: std::io::Error::other("disk gone") },
    ];
    for error in errors {
        let report = sweep_build_failure_report(&deleted(), &error);
        assert!(!report.lines.iter().any(|line| line.contains("nothing half-done")), "{report:?}");
        assert_eq!(report.exit as i32, 5);
    }
}
