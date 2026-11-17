//! after_publish_failure: after-publish's failure report (exit 5, S-71). Not failure_report, whose line "nothing
//! half-done is left" is false here: after_publish stops at a zone with the zones before it already moved.

use crate::report::{ExitCode, Report};

const VERB: &str = "after-publish";

/// Exit Failure, outcome "failed", `lines` as given, no owner, next a repeat of the same command, no fields.
pub fn after_publish_failure(lines: Vec<String>) -> Report {
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines,
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}
