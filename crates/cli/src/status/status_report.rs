//! status_report: the report of `mr status`, assembled from its parts; attempt_status_report builds on it for
//! an attempt folder.
//! status_report_tests covers a clean folder and one with every part as the whole Report, calling status_lines,
//! status_fields, status_next and status_outcome inside, and renders both forms, which runs oplog's exported, pure
//! JSON writer (so a unit test may use it).

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
