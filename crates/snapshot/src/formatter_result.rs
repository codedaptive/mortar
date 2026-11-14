//! FormatterResult: how a formatter run ended. A declaration only; decide_formatter_result_tests exercises it.

use std::time::Duration;

/// `Failed`, `SpawnFailed` and `TimedOut` are listed failures: the snapshot is still written, of the files as they are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatterResult {
    /// The formatter exited 0.
    Formatted,
    /// The formatter exited non-zero (`code` Some) or was killed by a signal (`code` None). `stderr` is the first
    /// FORMATTER_STDERR_CAP bytes read of its stderr (only what was read by the timeout, when a process it started
    /// still held the pipe open), read as UTF-8 with invalid bytes replaced.
    Failed { code: Option<i32>, stderr: String },
    /// The formatter could not be started, or waiting for it failed after it started, or reading its stderr failed
    /// where the read is joined (a read error is dropped on a timeout or a held-open pipe). `error` is
    /// the io::Error's Display text: io::Error is neither Clone nor Eq, so the error itself cannot be carried.
    SpawnFailed { error: String },
    /// The formatter was still running at the caller's timeout `limit`, so it was killed and reaped. `stderr` is the
    /// stderr read before the kill, its first FORMATTER_STDERR_CAP bytes, read as UTF-8 with invalid bytes replaced.
    TimedOut { limit: Duration, stderr: String },
}
