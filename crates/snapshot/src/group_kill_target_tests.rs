use crate::group_kill_target::group_kill_target;

#[test]
fn pid_zero_is_refused() {
    assert_eq!(group_kill_target(0), None);
}

#[test]
fn pid_one_is_refused() {
    assert_eq!(group_kill_target(1), None);
}

#[test]
fn pid_two_is_the_first_group_signalled() {
    assert_eq!(group_kill_target(2), Some(-2));
}

#[test]
fn an_ordinary_pid_is_negated() {
    assert_eq!(group_kill_target(4242), Some(-4242));
}

#[test]
fn i32_max_is_negated() {
    assert_eq!(group_kill_target(2147483647), Some(-2147483647));
}

#[test]
fn one_past_i32_max_is_refused() {
    assert_eq!(group_kill_target(2147483648), None);
}

#[test]
fn u32_max_is_refused() {
    assert_eq!(group_kill_target(u32::MAX), None);
}
