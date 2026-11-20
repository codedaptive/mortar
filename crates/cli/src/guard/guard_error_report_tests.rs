//! guard_error_report: ForeignHook, BadConfig and Io in the cli's own words. The Git case is an integration
//! test to write (see guard_error_report.rs).

use std::path::PathBuf;

use land::GuardError;

use super::guard_error_report;
use crate::report::{ExitCode, Report};

#[test]
fn a_foreign_hook_is_refused_with_nothing_written() {
    let error = GuardError::ForeignHook { path: PathBuf::from("/r/.git/hooks/pre-push") };
    assert_eq!(
        guard_error_report(&error),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Refused,
            outcome: "foreign-hook".to_string(),
            lines: vec![
                "/r/.git/hooks/pre-push is a hook Mortar did not write; Mortar never overwrites one".to_string(),
                "nothing was written".to_string(),
            ],
            owner: Some("the repository's own hooks".to_string()),
            next: "move /r/.git/hooks/pre-push aside or fold it into your own hook by hand, then mr guard install \
                   --actor <name>"
                .to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_guard_json_that_is_not_a_guard_config_is_refused_naming_it() {
    let error = GuardError::BadConfig {
        path: PathBuf::from("/r/.git/mortar/guard.json"),
        why: "it is not a JSON object".to_string(),
    };
    assert_eq!(
        guard_error_report(&error),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["/r/.git/mortar/guard.json is not a Mortar guard config: it is not a JSON object".to_string()],
            owner: Some("the Mortar guard".to_string()),
            next: "mr guard install --actor <name>".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn an_io_failure_is_a_failure_naming_the_path_and_the_error() {
    let error = GuardError::Io {
        path: PathBuf::from("/r/myhooks"),
        error: std::io::Error::other("disk full"),
    };
    assert_eq!(
        guard_error_report(&error),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "reading or writing /r/myhooks failed: disk full".to_string(),
                "what was already written is the guard's own and harmless; a repeat writes only what is missing or \
                 different"
                    .to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
