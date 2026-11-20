//! LeaderState: where a formatter's process group leader is, as run_formatter_child last saw it. A declaration only;
//! decide_group_kill_tests exercises it.

/// What decide_group_kill reads to tell whether the leader's group id is still reserved for its group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LeaderState {
    /// No exit seen.
    Running,
    /// An exit seen with WNOWAIT: the leader is a zombie, which still holds the group id.
    ExitedUnreaped,
    /// Its exit status was collected: the group id may be reused by another process.
    Reaped,
}
