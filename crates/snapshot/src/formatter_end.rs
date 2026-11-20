//! FormatterEnd: how run_formatter_child ends a formatter run. A declaration only; decide_formatter_end_tests
//! exercises it.

/// The end decide_formatter_end gives run_formatter_child once a poll's step is no longer Wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FormatterEnd {
    /// The leader is reaped and its stderr at EOF: join the reader and list its exit.
    Exited,
    /// The leader is reaped and its stderr is still open at the limit. Only a process outside its group (one that
    /// called setsid or setpgid) can still hold the pipe, since the group is gone on Linux and macOS. Leave the reader
    /// detached and list the leader's own exit with the stderr read so far.
    ExitedPipeHeld,
    /// The leader is still running at the limit; on Unix its group was killed this poll. Kill the leader too, reap it
    /// and list TimedOut.
    TimedOut,
}
