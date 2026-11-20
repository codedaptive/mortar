//! The shape every refusal of the landing verbs shares: exit 1, outcome "refused", no fields.

use crate::report::{ExitCode, Report};

/// Exit Refused, outcome "refused", the given lines, owner and next step, and no --json fields.
pub(crate) fn landing_refusal(verb: &str, lines: Vec<String>, owner: &str, next: String) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some(owner.to_string()),
        next,
        fields: Vec::new(),
    }
}
