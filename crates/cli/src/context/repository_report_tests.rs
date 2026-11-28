//! repository_report: outside a repository a harness verb is a usage error "not-a-repository", naming the
//! directory; git failing is a failure.

use std::path::PathBuf;

use gitio::GitError;

use super::{repository_report, ContextError};
use crate::report::{ExitCode, Report};

fn git_error(code: Option<i32>, stderr: &str) -> GitError {
    GitError {
        args: vec!["rev-parse".to_string(), "--path-format=absolute".to_string(), "--git-common-dir".to_string()],
        code,
        stderr: stderr.to_string(),
    }
}

#[test]
fn init_outside_a_repository_is_a_usage_error_naming_the_directory() {
    let error = ContextError::NotARepository {
        dir: PathBuf::from("/tmp/elsewhere"),
        error: git_error(Some(128), "fatal: not a git repository\n"),
    };
    assert_eq!(
        repository_report("init", &error),
        Report {
            verb: "init".to_string(),
            exit: ExitCode::Usage,
            outcome: "not-a-repository".to_string(),
            lines: vec![
                "/tmp/elsewhere is in no git repository".to_string(),
                "git rev-parse --path-format=absolute --git-common-dir: exit code 128: fatal: not a git repository"
                    .to_string(),
            ],
            owner: Some("whoever sets up Mortar in this repository".to_string()),
            next: "run mr init inside the git repository".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn git_failing_is_a_failure() {
    let report = repository_report("init", &ContextError::Git(git_error(None, "killed")));
    assert_eq!(report.exit as i32, 5);
    assert_eq!(report.outcome, "failed");
    assert_eq!(
        report.lines,
        vec![
            "git rev-parse --path-format=absolute --git-common-dir: no exit code: killed".to_string(),
        ]
    );
}
