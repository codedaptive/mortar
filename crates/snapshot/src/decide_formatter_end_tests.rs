use crate::decide_formatter_end::decide_formatter_end;
use crate::formatter_end::FormatterEnd;
use crate::formatter_wait::FormatterWait;

#[test]
fn wait_with_the_leader_reaped_does_not_end() {
    assert_eq!(decide_formatter_end(FormatterWait::Wait, true), None);
}

#[test]
fn wait_with_the_leader_not_reaped_does_not_end() {
    assert_eq!(decide_formatter_end(FormatterWait::Wait, false), None);
}

#[test]
fn done_is_exited() {
    assert_eq!(decide_formatter_end(FormatterWait::Done, true), Some(FormatterEnd::Exited));
}

#[test]
fn kill_with_the_leader_reaped_is_exited_with_the_pipe_held() {
    assert_eq!(decide_formatter_end(FormatterWait::Kill, true), Some(FormatterEnd::ExitedPipeHeld));
}

#[test]
fn kill_with_the_leader_not_reaped_is_timed_out() {
    assert_eq!(decide_formatter_end(FormatterWait::Kill, false), Some(FormatterEnd::TimedOut));
}
