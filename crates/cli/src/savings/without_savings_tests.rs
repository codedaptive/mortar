//! without_savings: every Saved field goes, all else stays in order.

use super::{without_savings, Savings};
use crate::report::{ExitCode, OutValue, Report};

fn report(fields: Vec<(String, OutValue)>) -> Report {
    Report {
        verb: "status".to_string(),
        exit: ExitCode::Done,
        outcome: "clean".to_string(),
        lines: vec!["a".to_string()],
        owner: Some("o".to_string()),
        next: "x".to_string(),
        fields,
    }
}

fn saved(key: &str) -> (String, OutValue) {
    (key.to_string(), OutValue::Saved(Savings { baseline: "b".to_string(), baseline_chars: 1 }))
}

fn text(key: &str) -> (String, OutValue) {
    (key.to_string(), OutValue::Text(key.to_string()))
}

#[test]
fn removes_every_saved_field() {
    let given = report(vec![saved("s1"), text("a"), saved("s2"), text("b")]);
    assert_eq!(without_savings(given), report(vec![text("a"), text("b")]));
}

#[test]
fn a_report_without_savings_is_unchanged() {
    assert_eq!(without_savings(report(vec![text("a")])), report(vec![text("a")]));
}
