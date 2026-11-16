use super::step_verdict;
use crate::report::Verdict;

#[test]
#[ignore = "acceptance: make accept"]
fn a_successful_removal_keeps_the_body_verdict() {
    assert_eq!(step_verdict(Verdict::Pass, true), Verdict::Pass);
    assert_eq!(step_verdict(Verdict::Fail, true), Verdict::Fail);
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_failed_removal_fails_the_step() {
    assert_eq!(step_verdict(Verdict::Pass, false), Verdict::Fail);
    assert_eq!(step_verdict(Verdict::Fail, false), Verdict::Fail);
}
