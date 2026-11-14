//! The report of a verb that git or the disk failed (exit 5).

use super::{ExitCode, Report};

/// Exit Failure, outcome "failed": `what` failed, then that nothing half-done is left, so the next step is the
/// same command again (S-71).
pub fn failure_report(verb: &str, what: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![what.to_string(), "nothing half-done is left".to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}
