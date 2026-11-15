//! status_report: the report of `mortar status`, assembled from its parts.
//! No sibling unit test: it calls status_lines, status_fields, status_next and status_outcome, so its tests are the
//! ZIT crates/cli/tests/status_report_tests.rs, which also renders both forms (S-104).

use super::{status_fields, status_lines, status_next, status_outcome, StatusState};
use crate::report::{ExitCode, Report};

/// Exit Done for every state: a state that was read is a finished answer, whatever it holds. No owner; the
/// outcome, lines, next step and fields are the parts' answers for `state`.
pub fn status_report(state: &StatusState) -> Report {
    Report {
        verb: "status".to_string(),
        exit: ExitCode::Done,
        outcome: status_outcome(state).to_string(),
        lines: status_lines(state),
        owner: None,
        next: status_next(state),
        fields: status_fields(state),
    }
}
