//! rereview_report: land's diff as the report's lines, and the empty diff.

use gitio::Oid;

use super::rereview_report;
use crate::report::{ExitCode, OutValue, Report};

const REVIEWED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn report(diff: &str) -> Report {
    rereview_report(&Oid::parse(REVIEWED).unwrap(), &Oid::parse(NEW).unwrap(), diff)
}

fn expected(lines: &[&str]) -> Report {
    Report {
        verb: "rereview".to_string(),
        exit: ExitCode::Done,
        outcome: "rereviewed".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: None,
        next: "hand this diff to the reviewer of 1111111111111111111111111111111111111111".to_string(),
        fields: vec![
            ("reviewed".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
            ("new".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
        ],
    }
}

#[test]
fn the_diffs_lines_are_the_reports_lines_in_order() {
    let diff = "1:  aaaaaaa ! 1:  bbbbbbb squash\n    @@ src/a/z (new)\n    ++z content\n";
    assert_eq!(report(diff), expected(&["1:  aaaaaaa ! 1:  bbbbbbb squash", "    @@ src/a/z (new)", "    ++z content"]));
}

#[test]
fn a_diff_without_a_final_newline_keeps_its_last_line() {
    assert_eq!(report("first\nsecond"), expected(&["first", "second"]));
}

#[test]
fn an_empty_diff_is_one_line_saying_so() {
    assert_eq!(report(""), expected(&["the range-diff is empty"]));
}
