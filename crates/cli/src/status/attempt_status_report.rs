//! attempt_status_report: the report of `mr status` in attempt folder k.

use super::{attempt_next, status_report, StatusState};
use crate::report::{OutValue, Report};

/// status_report for `state`, with the line `attempt folder {k} on branch {branch}` right after the first line,
/// next from attempt_next, and the fields `attempt` (k) and `branch` appended. Exit, outcome, verb and owner are
/// status_report's.
pub fn attempt_status_report(k: u32, branch: &str, state: &StatusState) -> Report {
    let mut report = status_report(state);
    report.lines.insert(1, format!("attempt folder {k} on branch {branch}"));
    report.next = attempt_next(state);
    report.fields.push(("attempt".to_string(), OutValue::Number(u64::from(k))));
    report.fields.push(("branch".to_string(), OutValue::Text(branch.to_string())));
    report
}
