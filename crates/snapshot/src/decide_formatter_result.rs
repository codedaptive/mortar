//! decide_formatter_result: how a formatter run ended, from its exit code and stderr or its spawn error.

use crate::formatter_result::FormatterResult;
use crate::formatter_stderr_cap::FORMATTER_STDERR_CAP;
use std::io;

/// Exit 0 is Formatted, its stderr dropped (a formatter may warn and still succeed). Any other exit, or none (killed
/// by a signal), is Failed with the first FORMATTER_STDERR_CAP bytes of stderr, read lossy as UTF-8 (the cut may
/// split a character; it shows as a replacement character). An io error (spawn, stderr read or wait) is SpawnFailed
/// with its text. Failed and SpawnFailed are listed failures: the snapshot goes on.
pub(crate) fn decide_formatter_result(exit: Result<(Option<i32>, Vec<u8>), io::Error>) -> FormatterResult {
    match exit {
        Ok((Some(0), _)) => FormatterResult::Formatted,
        Ok((code, stderr)) => FormatterResult::Failed {
            code,
            stderr: String::from_utf8_lossy(&stderr[..stderr.len().min(FORMATTER_STDERR_CAP)]).into_owned(),
        },
        Err(e) => FormatterResult::SpawnFailed { error: e.to_string() },
    }
}
