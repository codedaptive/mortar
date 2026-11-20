//! GroupRunEnd: how a run_in_group command ended. A declaration only; decide_group_run_end_tests exercises it.

use std::time::Duration;

/// The end of one command run_in_group ran (on Unix as the leader of its own process group).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupRunEnd {
    /// The command exited by itself: `code` is its exit code, None when killed by a signal. `stderr` is the first
    /// FORMATTER_STDERR_CAP (4096) bytes read of its stderr (only what was read by the limit, when a process it
    /// started still held the pipe open: on Linux and macOS, only one outside its process group can), read as UTF-8
    /// with invalid bytes replaced. Exit 0 keeps its stderr too.
    Exited { code: Option<i32>, stderr: String },
    /// The command was still running at `limit`, so its process group (on Unix) and it were killed and it was
    /// reaped. `stderr` is the stderr read before the kill, its first FORMATTER_STDERR_CAP (4096) bytes, read as UTF-8
    /// with invalid bytes replaced.
    TimedOut { limit: Duration, stderr: String },
    /// The command could not be started, or waiting for it failed after it started, or reading its stderr failed
    /// where the read is joined (a read error is dropped on a timeout or a held-open pipe). `error` is the io::Error's
    /// Display text: io::Error is neither Clone nor Eq, so the error itself cannot be carried.
    SpawnFailed { error: String },
}
