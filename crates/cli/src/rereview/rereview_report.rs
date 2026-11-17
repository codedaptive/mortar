//! The report of a `mortar rereview` that land answered with its re-review diff.

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0 Done, outcome "rereviewed": one line per line of `diff`, land's answer, in order ("the range-diff is
/// empty" when it has none), no owner, next handing the diff to the reviewer of `reviewed`. Fields: reviewed and
/// new, each the commit id.
pub fn rereview_report(reviewed: &Oid, new: &Oid, diff: &str) -> Report {
    let mut lines: Vec<String> = diff.lines().map(str::to_string).collect();
    if lines.is_empty() {
        lines.push("the range-diff is empty".to_string());
    }
    Report {
        verb: "rereview".to_string(),
        exit: ExitCode::Done,
        outcome: "rereviewed".to_string(),
        lines,
        owner: None,
        next: format!("hand this diff to the reviewer of {}", reviewed.as_str()),
        fields: vec![
            ("reviewed".to_string(), OutValue::Text(reviewed.as_str().to_string())),
            ("new".to_string(), OutValue::Text(new.as_str().to_string())),
        ],
    }
}
