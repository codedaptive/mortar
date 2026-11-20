//! rereview_report: land's text as the report's lines, the empty text, and the outcome, next and same_patch field
//! for a different patch and for the same patch.

use gitio::Oid;
use land::RereviewAnswer;

use super::rereview_report;
use crate::report::{ExitCode, OutValue, Report};

const REVIEWED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn report(text: &str, same_patch: bool) -> Report {
    let answer = RereviewAnswer { text: text.to_string(), same_patch };
    rereview_report(&Oid::parse(REVIEWED).unwrap(), &Oid::parse(NEW).unwrap(), &answer)
}

fn expected(outcome: &str, next: &str, same_patch: bool, lines: &[&str]) -> Report {
    Report {
        verb: "rereview".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: None,
        next: next.to_string(),
        fields: vec![
            ("reviewed".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
            ("new".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
            ("same_patch".to_string(), OutValue::Bool(same_patch)),
        ],
    }
}

fn rereviewed(lines: &[&str]) -> Report {
    expected("rereviewed", "hand this diff to the reviewer of 1111111111111111111111111111111111111111", false, lines)
}

fn same_patch(lines: &[&str]) -> Report {
    expected(
        "same-patch",
        "2222222222222222222222222222222222222222 carries the same patch as \
         1111111111111111111111111111111111111111; whether the review of \
         1111111111111111111111111111111111111111 still holds on the new base is the caller's to decide",
        true,
        lines,
    )
}

#[test]
fn the_diffs_lines_are_the_reports_lines_in_order() {
    let diff = "1:  aaaaaaa ! 1:  bbbbbbb squash\n    @@ src/a/z (new)\n    ++z content\n";
    assert_eq!(
        report(diff, false),
        rereviewed(&["1:  aaaaaaa ! 1:  bbbbbbb squash", "    @@ src/a/z (new)", "    ++z content"])
    );
}

#[test]
fn a_diff_without_a_final_newline_keeps_its_last_line() {
    assert_eq!(report("first\nsecond", false), rereviewed(&["first", "second"]));
}

#[test]
fn an_empty_diff_is_one_line_saying_so() {
    assert_eq!(report("", false), rereviewed(&["the range-diff is empty"]));
}

#[test]
fn the_same_patch_is_outcome_same_patch_with_the_diffs_lines_and_the_field_true() {
    let diff = "1:  aaaaaaa = 1:  bbbbbbb squash\n";
    assert_eq!(report(diff, true), same_patch(&["1:  aaaaaaa = 1:  bbbbbbb squash"]));
}

#[test]
fn the_same_patch_keeps_every_line_in_order() {
    assert_eq!(report("first\nsecond", true), same_patch(&["first", "second"]));
}

#[test]
fn the_same_patch_with_an_empty_text_is_one_line_saying_so() {
    assert_eq!(report("", true), same_patch(&["the range-diff is empty"]));
}

#[test]
fn the_same_patch_never_hands_the_diff_to_a_reviewer() {
    let next = report("1:  aaaaaaa = 1:  bbbbbbb squash\n", true).next;
    assert!(!next.contains("hand this diff to the reviewer"));
    assert!(!next.contains("review holds"));
}
