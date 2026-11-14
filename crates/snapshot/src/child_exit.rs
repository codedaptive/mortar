//! ChildExit: how a formatter child ended, as run_formatter_child saw it. A declaration only;
//! decide_formatter_result_tests exercises it.

use std::time::Duration;

/// The raw end of one formatter child, before decide_formatter_result reads it.
#[derive(Debug)]
pub(crate) enum ChildExit {
    /// The child exited by itself: `code` is its exit code, None when killed by a signal; `stderr` is what was read
    /// of its stderr, up to FORMATTER_STDERR_CAP bytes: all of it, unless a process the child started still held the
    /// pipe open at the limit (on Linux and macOS only one outside the child's process group can).
    Exited { code: Option<i32>, stderr: Vec<u8> },
    /// The child was still running at `limit`, so it and its process group were killed and it was reaped; `stderr` is
    /// what was read before the kill.
    TimedOut { limit: Duration, stderr: Vec<u8> },
}
