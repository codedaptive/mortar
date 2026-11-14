use crate::decide_group_kill::decide_group_kill;
use crate::formatter_wait::FormatterWait::{Done, Kill, Wait};
use crate::leader_state::LeaderState::{ExitedUnreaped, Reaped, Running};

#[test]
fn running_not_killed_waiting_is_not_signalled() {
    assert!(!decide_group_kill(Running, false, Wait));
}

#[test]
fn running_not_killed_at_the_limit_is_signalled() {
    assert!(decide_group_kill(Running, false, Kill));
}

#[test]
fn running_not_killed_done_is_not_signalled() {
    assert!(!decide_group_kill(Running, false, Done));
}

#[test]
fn running_already_killed_waiting_is_not_signalled() {
    assert!(!decide_group_kill(Running, true, Wait));
}

#[test]
fn running_already_killed_at_the_limit_is_not_signalled_again() {
    assert!(!decide_group_kill(Running, true, Kill));
}

#[test]
fn running_already_killed_done_is_not_signalled() {
    assert!(!decide_group_kill(Running, true, Done));
}

#[test]
fn exited_unreaped_not_killed_waiting_is_signalled() {
    assert!(decide_group_kill(ExitedUnreaped, false, Wait));
}

#[test]
fn exited_unreaped_not_killed_at_the_limit_is_signalled() {
    assert!(decide_group_kill(ExitedUnreaped, false, Kill));
}

#[test]
fn exited_unreaped_not_killed_done_is_signalled() {
    assert!(decide_group_kill(ExitedUnreaped, false, Done));
}

#[test]
fn exited_unreaped_already_killed_waiting_is_not_signalled() {
    assert!(!decide_group_kill(ExitedUnreaped, true, Wait));
}

#[test]
fn exited_unreaped_already_killed_at_the_limit_is_not_signalled() {
    assert!(!decide_group_kill(ExitedUnreaped, true, Kill));
}

#[test]
fn exited_unreaped_already_killed_done_is_not_signalled() {
    assert!(!decide_group_kill(ExitedUnreaped, true, Done));
}

#[test]
fn reaped_not_killed_waiting_is_not_signalled() {
    assert!(!decide_group_kill(Reaped, false, Wait));
}

#[test]
fn reaped_not_killed_at_the_limit_is_not_signalled() {
    assert!(!decide_group_kill(Reaped, false, Kill));
}

#[test]
fn reaped_not_killed_done_is_not_signalled() {
    assert!(!decide_group_kill(Reaped, false, Done));
}

#[test]
fn reaped_already_killed_waiting_is_not_signalled() {
    assert!(!decide_group_kill(Reaped, true, Wait));
}

#[test]
fn reaped_already_killed_at_the_limit_is_not_signalled() {
    assert!(!decide_group_kill(Reaped, true, Kill));
}

#[test]
fn reaped_already_killed_done_is_not_signalled() {
    assert!(!decide_group_kill(Reaped, true, Done));
}
