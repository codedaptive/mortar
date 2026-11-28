//! The report of a harness verb whose directory's git repository cannot be found.

use super::{context_report, ContextError};
use crate::report::Report;

/// NotARepository: context_report's answer (usage, exit 2, outcome "not-a-repository", naming the directory and
/// git's error, next "run mr <verb> inside the git repository"), owned by whoever sets up Mortar. Every other
/// error (Git, from repository_error) reports as context_report does: git failing is a failure (exit 5).
pub(crate) fn repository_report(verb: &str, error: &ContextError) -> Report {
    match error {
        ContextError::NotARepository { .. } => Report {
            owner: Some("whoever sets up Mortar in this repository".to_string()),
            ..context_report(verb, error)
        },
        other => context_report(verb, other),
    }
}
