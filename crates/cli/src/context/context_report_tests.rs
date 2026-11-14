//! context_report: not in a repository or not a folder is refused "not-a-folder"; a bad record is refused naming
//! the file; an unreadable record, any other folder error and git failing are failures.

use std::path::PathBuf;

use folders::record::RecordError;
use folders::FolderError;
use gitio::GitError;

use super::{context_report, ContextError};
use crate::report::{ExitCode, Report};

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
            next: "run mortar save inside a folder Mortar made".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_worktree_without_a_record_is_not_a_folder_naming_it() {
    let report = context_report("join", &ContextError::NotAFolder { toplevel: PathBuf::from("/r/main") });
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "not-a-folder");
    assert_eq!(
        report.lines,
        vec!["this directory is not in a Mortar folder: the worktree /r/main has no Mortar record".to_string()]
    );
    assert_eq!(report.owner.as_deref(), Some("the harness that makes Mortar folders"));
    assert_eq!(report.next, "run mortar join inside a folder Mortar made");
}

#[test]
fn a_record_mortar_did_not_write_is_refused_naming_the_file() {
    let error = ContextError::Record(FolderError::Record {
        path: PathBuf::from("/r/.git/worktrees/a/mortar-folder.json"),
        error: RecordError { reason: "zone is missing".to_string() },
    });
    let report = context_report("save", &error);
    assert_eq!(report.verb, "save");
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "refused");
    assert_eq!(
        report.lines[0],
        "the folder record /r/.git/worktrees/a/mortar-folder.json is not one Mortar writes: zone is missing"
    );
    assert_eq!(report.owner.as_deref(), Some("the harness that makes Mortar folders"));
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
