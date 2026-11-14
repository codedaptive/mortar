//! FormatterResult: how a formatter run ended. A declaration only; decide_formatter_result_tests exercises it.

/// `Failed` and `SpawnFailed` are listed failures: the snapshot is still written, of the files as they are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatterResult {
    /// The formatter exited 0.
    Formatted,
    /// The formatter exited non-zero (`code` Some) or was killed by a signal (`code` None). `stderr` is its first
    /// FORMATTER_STDERR_CAP bytes, read as UTF-8 with invalid bytes replaced.
    Failed { code: Option<i32>, stderr: String },
    /// The formatter could not be started, or reading its stderr or waiting for it failed after it started. `error` is
    /// the io::Error's Display text: io::Error is neither Clone nor Eq, so the error itself cannot be carried.
    SpawnFailed { error: String },
}
