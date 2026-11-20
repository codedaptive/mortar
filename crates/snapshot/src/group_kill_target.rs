//! group_kill_target: the pid argument kill(2) needs to signal a formatter's whole process group.

/// The negated `leader_pid`, which makes kill(2) signal every process in the group the leader leads (the formatter
/// is spawned with process_group(0), so its group id is its pid). None when no group may be signalled: 0, because
/// kill(0) signals mortar's own group; 1, because kill(-1) signals every process mortar may signal; and anything
/// over i32::MAX, because it does not fit kill's pid_t.
pub(crate) fn group_kill_target(leader_pid: u32) -> Option<i32> {
    match leader_pid {
        0 | 1 => None,
        pid => i32::try_from(pid).ok().map(|pid| -pid),
    }
}
