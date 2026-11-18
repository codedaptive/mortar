//! hooks_path_report: a relative core.hooksPath refused naming the value, and a git failure in the cli's own words.

use std::path::PathBuf;

use gitio::GitError;
use land::HooksPathError;

use super::hooks_path_report;
use crate::report::{ExitCode, Report};

#[test]
fn a_relative_hooks_path_is_refused_naming_it_with_nothing_written() {
    let error = HooksPathError::Relative { value: PathBuf::from("myhooks") };
    assert_eq!(
        hooks_path_report(&error),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "core.hooksPath is \"myhooks\", not an absolute path: git resolves a relative hooks path in each \
                 folder, so one install cannot guard them all"
                    .to_string(),
                "nothing was written".to_string(),
            ],
            owner: Some("the repository's git config".to_string()),
            next: "unset core.hooksPath or set it to an absolute path, then mr guard install --actor <name>"
                .to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_dot_relative_hooks_path_is_refused_naming_it() {
    let error = HooksPathError::Relative { value: PathBuf::from(".husky") };
    assert_eq!(
        hooks_path_report(&error),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "core.hooksPath is \".husky\", not an absolute path: git resolves a relative hooks path in each \
                 folder, so one install cannot guard them all"
                    .to_string(),
                "nothing was written".to_string(),
            ],
            owner: Some("the repository's git config".to_string()),
            next: "unset core.hooksPath or set it to an absolute path, then mr guard install --actor <name>"
                .to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_git_failure_is_a_failure_in_the_clis_words() {
    let error = HooksPathError::Git(GitError::refusal(vec!["config".to_string()], "fatal: gitio text 7731"));
    assert_eq!(
        hooks_path_report(&error),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec!["git could not read core.hooksPath".to_string(), "nothing half-done is left".to_string()],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
