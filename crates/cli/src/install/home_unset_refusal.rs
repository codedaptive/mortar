//! home_unset_refusal: The refusal of install or uninstall when HOME is unset or empty.
//! Pure: builds the report from its arguments; home_unset_refusal_tests pins its text.

use crate::report::{ExitCode, Report};

/// Refused (exit 3) for `verb`: HOME is not set or is empty, and nothing changed; `owner` is who must set it.
pub fn home_unset_refusal(verb: &str, owner: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec!["HOME is not set or is empty".to_string(), "nothing changed".to_string()],
        owner: Some(owner.to_string()),
        next: "set HOME, then repeat".to_string(),
        fields: Vec::new(),
    }
}
