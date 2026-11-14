use crate::decide_formatter_wait::decide_formatter_wait;
use crate::formatter_wait::FormatterWait;
use std::time::Duration;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn just_started_inside_the_limit_waits() {
    assert_eq!(decide_formatter_wait(ms(0), ms(5000), false), FormatterWait::Wait);
}

#[test]
fn one_millisecond_before_the_limit_waits() {
    assert_eq!(decide_formatter_wait(ms(4999), ms(5000), false), FormatterWait::Wait);
}

#[test]
fn running_at_the_limit_is_killed() {
    assert_eq!(decide_formatter_wait(ms(5000), ms(5000), false), FormatterWait::Kill);
}

#[test]
fn running_past_the_limit_is_killed() {
    assert_eq!(decide_formatter_wait(ms(6000), ms(5000), false), FormatterWait::Kill);
}

#[test]
fn an_exit_past_the_limit_is_done() {
    assert_eq!(decide_formatter_wait(ms(6000), ms(5000), true), FormatterWait::Done);
}

#[test]
fn an_exit_inside_the_limit_is_done() {
    assert_eq!(decide_formatter_wait(ms(0), ms(5000), true), FormatterWait::Done);
}

#[test]
fn a_zero_limit_kills_a_run_not_exited() {
    assert_eq!(decide_formatter_wait(ms(0), ms(0), false), FormatterWait::Kill);
}
