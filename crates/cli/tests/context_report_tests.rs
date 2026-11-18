//! context_report: outside a repository is refused "not-a-folder" with git's line; an unreadable record, any other
//! folder error and git failing are failures. A worktree without a record and a bad record are the ZUTs in
//! src/context/context_report_tests.rs.
//! ZIT (S-79 as replaced: the line is the file): it calls report::failure_report for real.

use std::path::PathBuf;

use folders::FolderError;
use gitio::GitError;

use cli::{context_report, ContextError, ExitCode, Report};

fn git_error(code: Option<i32>, stderr: &str) -> GitError {
    GitError { args: vec!["rev-parse".to_string(), "--show-toplevel".to_string()], code, stderr: stderr.to_string() }
}

#[test]
fn outside_a_repository_is_not_a_folder() {
    let error = ContextError::NotARepository {
        dir: PathBuf::from("/tmp/elsewhere"),
        error: git_error(Some(128), "fatal: not a git repository\n"),
    };
    assert_eq!(
        context_report("save", &error),
        Report {
            verb: "save".to_string(),
            exit: ExitCode::Refused,
            outcome: "not-a-folder".to_string(),
            lines: vec![
                "this directory is not in a Mortar folder: /tmp/elsewhere is in no git repository".to_string(),
                "git rev-parse --show-toplevel: exit code 128: fatal: not a git repository".to_string(),
            ],
            owner: Some("the harness that makes Mortar folders".to_string()),
            next: "run mr save inside a folder Mortar made".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_record_that_cannot_be_read_is_a_failure() {
    let error = ContextError::Record(FolderError::Io {
        path: PathBuf::from("/r/.git/mortar-folder.json"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    });
    let report = context_report("save", &error);
    assert_eq!(report.exit, ExitCode::Failure);
    assert_eq!(report.outcome, "failed");
    assert_eq!(report.next, "repeat the same command");
}

#[test]
fn another_folder_error_is_a_failure() {
    let error = ContextError::Record(FolderError::BranchExists { branch: "req/a/b".to_string() });
    assert_eq!(context_report("save", &error).exit, ExitCode::Failure);
}

#[test]
fn git_failing_is_a_failure_naming_the_run() {
    let report = context_report("save", &ContextError::Git(git_error(None, "killed")));
    assert_eq!(report.exit as i32, 5);
    assert_eq!(
        report.lines,
        vec![
            "git rev-parse --show-toplevel: no exit code: killed".to_string(),
            "nothing half-done is left".to_string(),
        ]
    );
}
