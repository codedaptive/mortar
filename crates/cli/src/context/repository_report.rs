//! The report of a harness verb whose directory's git repository cannot be found.

use super::{context_report, ContextError};
use crate::report::{ExitCode, Report};

/// NotARepository: refused (exit 1), outcome "not-a-repository", naming the directory and git's error, owned by
/// whoever sets up Mortar, next "run mr <verb> inside the git repository". Every other error (Git, from
/// repository_error) reports as context_report does: git failing is a failure (exit 5).
pub(crate) fn repository_report(verb: &str, error: &ContextError) -> Report {
    match error {
        ContextError::NotARepository { dir, error } => Report {
            verb: verb.to_string(),
            exit: ExitCode::Refused,
            outcome: "not-a-repository".to_string(),
            lines: vec![format!("{} is in no git repository", dir.display()), error.to_string()],
            owner: Some("whoever sets up Mortar in this repository".to_string()),
            next: format!("run mr {verb} inside the git repository"),
            fields: Vec::new(),
        },
        other => context_report(verb, other),
    }
}
