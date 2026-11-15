//! need_report: the report of a path added to a request folder by `mortar need` (spec 02).

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0. Outcome "needed" with `added <path> to this folder`, or "already-done" with `<path> was already needed by
/// this folder` when the record named it before (`already`); then, when the path is outside the work order (`gap`),
/// `<path> is outside the work order: recorded as a gap`. Fields path and gap; `next` is the step status gives.
pub fn need_report(path: &str, already: bool, gap: bool, next: String) -> Report {
    let (outcome, first) = if already {
        ("already-done", format!("{path} was already needed by this folder"))
    } else {
        ("needed", format!("added {path} to this folder"))
    };
    let mut lines = vec![first];
    if gap {
        lines.push(format!("{path} is outside the work order: recorded as a gap"));
    }
    Report {
        verb: "need".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next,
        fields: vec![("path".to_string(), OutValue::Text(path.to_string())), ("gap".to_string(), OutValue::Bool(gap))],
    }
}
