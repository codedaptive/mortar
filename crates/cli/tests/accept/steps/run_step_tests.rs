use std::ffi::OsString;

use super::{run_step, Gate, StepBody, StepSpec};
use crate::report::{Checks, Line, Verdict};
use crate::world::World;

fn unset(_: &str) -> Option<OsString> {
    None
}

fn spec(body: StepBody) -> StepSpec {
    StepSpec { number: 9, title: "a test step", order: "ACCEPT-0", gate: Gate::Always, body: Some(body) }
}

fn line(ok: bool, did: &str, saw: &str) -> Line {
    Line::Check { ok, did: did.to_string(), saw: saw.to_string() }
}

fn nothing(_: &World, _: &mut Checks) {}

fn one_ok(_: &World, checks: &mut Checks) {
    checks.check("one check", true, "held");
}

fn one_failed(_: &World, checks: &mut Checks) {
    checks.check("one check", false, "did not hold");
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_body_that_checks_nothing_fails_even_when_the_removal_succeeds() {
    let block = run_step(&spec(nothing), &unset);
    assert_eq!(block.verdict, Verdict::Fail);
    assert_eq!(block.lines, vec![line(true, "removed the throwaway repositories", "removed")]);
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_body_with_one_check_that_held_passes() {
    let block = run_step(&spec(one_ok), &unset);
    assert_eq!(block.verdict, Verdict::Pass);
    assert_eq!(
        block.lines,
        vec![line(true, "one check", "held"), line(true, "removed the throwaway repositories", "removed")]
    );
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_body_with_one_failed_check_fails() {
    let block = run_step(&spec(one_failed), &unset);
    assert_eq!(block.verdict, Verdict::Fail);
    assert_eq!(
        block.lines,
        vec![line(false, "one check", "did not hold"), line(true, "removed the throwaway repositories", "removed")]
    );
}
