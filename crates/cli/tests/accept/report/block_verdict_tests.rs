use super::{block_verdict, Line, Verdict};

fn check(ok: bool) -> Line {
    Line::Check { ok, did: "did".to_string(), saw: "saw".to_string() }
}

#[test]
fn every_check_held_is_pass() {
    assert_eq!(block_verdict(&[check(true), Line::Log("x".to_string()), check(true)], None), Verdict::Pass);
}

#[test]
fn one_check_failed_is_fail() {
    assert_eq!(block_verdict(&[check(true), check(false), check(true)], None), Verdict::Fail);
}

#[test]
fn no_check_is_fail_even_with_log_lines() {
    assert_eq!(block_verdict(&[], None), Verdict::Fail);
    assert_eq!(block_verdict(&[Line::Log("only a log".to_string())], None), Verdict::Fail);
}

#[test]
fn a_panic_is_fail_even_when_every_check_held() {
    assert_eq!(block_verdict(&[check(true)], Some("boom")), Verdict::Fail);
}
