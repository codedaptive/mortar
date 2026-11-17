use super::CaseLines;
use crate::report::{Checks, Line};

fn line(ok: bool, did: &str, saw: &str) -> Line {
    Line::Check { ok, did: did.to_string(), saw: saw.to_string() }
}

#[test]
fn a_case_has_its_own_checks_then_the_five_consistency_items() {
    let lines = CaseLines::new(2);
    assert_eq!(lines.own, 4);
    assert_eq!(lines.titles.len(), 9);
    assert_eq!(lines.titles[1], "3. before: mr land <same commit> again: landed");
    assert_eq!(lines.titles[4], "3. before: consistent: git fsck clean");
    assert_eq!(lines.titles[8], "3. before: consistent: repeating the finished command once more moves nothing");
}

#[test]
fn checks_take_the_titles_in_order_and_not_run_fills_the_rest_of_the_case() {
    let mut checks = Checks::new();
    let mut lines = CaseLines::new(0);
    lines.check(&mut checks, true, "killed by signal 9");
    lines.check(&mut checks, false, "zone/a moved");
    lines.not_run(&mut checks, "the kill failed", false);
    lines.check(&mut checks, true, "exit 0");
    let recorded = checks.lines();
    assert_eq!(recorded.len(), 6);
    assert_eq!(
        recorded[0],
        line(
            true,
            "1. mr join in r1 (a fast-forward), killed before its ref transaction: the op log's open intent is that join",
            "killed by signal 9"
        )
    );
    assert_eq!(recorded[1], line(false, "1. zone/a unmoved", "zone/a moved"));
    assert_eq!(recorded[2], line(false, "1. mr join in r1 again finishes", "not run: the kill failed"));
    assert_eq!(recorded[4], line(false, "1. the zone moved once", "not run: the kill failed"));
    assert_eq!(recorded[5], line(true, "1. consistent: git fsck clean", "exit 0"));
}

#[test]
fn not_run_with_consistent_fills_every_remaining_title() {
    let mut checks = Checks::new();
    let mut lines = CaseLines::new(4);
    lines.not_run(&mut checks, "case 3 failed", true);
    let recorded = checks.lines();
    assert_eq!(recorded.len(), 10);
    assert_eq!(recorded[1], line(false, "4. the landing state says a publish was interrupted", "not run: case 3 failed"));
    assert_eq!(
        recorded[9],
        line(false, "4. consistent: repeating the finished command once more moves nothing", "not run: case 3 failed")
    );
}

#[test]
fn a_line_past_the_last_title_still_shows() {
    let mut checks = Checks::new();
    let mut lines = CaseLines::new(1);
    lines.not_run(&mut checks, "x", true);
    lines.check(&mut checks, true, "extra");
    assert_eq!(checks.lines().last(), Some(&line(true, "a line past the case's checks", "extra")));
}
