//! decide_formatter_result: how a formatter run ended, from how its child ended or its spawn error.

use crate::child_exit::ChildExit;
use crate::formatter_result::FormatterResult;
use crate::formatter_stderr_cap::FORMATTER_STDERR_CAP;
use std::io;

/// Exit 0 is Formatted, its stderr dropped (a formatter may warn and still succeed). Any other exit, or none (killed
/// by a signal), is Failed with its code and stderr. A child killed at its timeout is TimedOut with the limit and the
/// stderr read before the kill, whatever it would have exited with. An io error (spawn, wait, or a joined stderr read)
/// is SpawnFailed with its text. Failed, TimedOut and SpawnFailed are listed failures: the snapshot goes on.
pub(crate) fn decide_formatter_result(exit: io::Result<ChildExit>) -> FormatterResult {
    match exit {
        Ok(ChildExit::Exited { code: Some(0), .. }) => FormatterResult::Formatted,
        Ok(ChildExit::Exited { code, stderr }) => FormatterResult::Failed { code, stderr: capped_text(&stderr) },
        Ok(ChildExit::TimedOut { limit, stderr }) => FormatterResult::TimedOut { limit, stderr: capped_text(&stderr) },
        Err(e) => FormatterResult::SpawnFailed { error: e.to_string() },
    }
}

/// The first FORMATTER_STDERR_CAP bytes of `stderr`, read lossy as UTF-8 (the cut may split a character; it shows as
/// a replacement character).
fn capped_text(stderr: &[u8]) -> String {
    String::from_utf8_lossy(&stderr[..stderr.len().min(FORMATTER_STDERR_CAP)]).into_owned()
}
