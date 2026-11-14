//! need_report: a new path is "needed", a path the record already named is "already-done"; a gap adds its line;
//! every report is exit 0 with the path and gap fields and the next step it is given.

use super::need_report;
use crate::report::{ExitCode, OutValue, Report};

const GAP_LINE: &str = "kit/b is outside the work order: recorded as a gap";

fn expected(outcome: &str, lines: &[&str], gap: bool, next: &str) -> Report {
    Report {
        verb: "need".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: None,
        next: next.to_string(),
        fields: vec![
            ("path".to_string(), OutValue::Text("kit/b".to_string())),
            ("gap".to_string(), OutValue::Bool(gap)),
        ],
    }
}

#[test]
fn a_new_path_in_the_work_order_is_needed_with_one_line() {
    assert_eq!(
        need_report("kit/b", false, false, "mortar join".to_string()),
        expected("needed", &["added kit/b to this folder"], false, "mortar join"),
    );
}

#[test]
fn a_new_path_outside_the_work_order_says_it_is_recorded_as_a_gap() {
    assert_eq!(
        need_report("kit/b", false, true, "mortar save \"<sentence>\"".to_string()),
        expected("needed", &["added kit/b to this folder", GAP_LINE], true, "mortar save \"<sentence>\""),
    );
}

#[test]
fn a_path_already_needed_is_already_done() {
    assert_eq!(
        need_report("kit/b", true, false, "wait for the zone's tests".to_string()),
        expected("already-done", &["kit/b was already needed by this folder"], false, "wait for the zone's tests"),
    );
}

#[test]
fn a_path_already_needed_outside_the_work_order_still_says_gap() {
    assert_eq!(
        need_report("kit/b", true, true, "mortar join".to_string()),
        expected("already-done", &["kit/b was already needed by this folder", GAP_LINE], true, "mortar join"),
    );
}
