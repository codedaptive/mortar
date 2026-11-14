//! decide_group_run_end: how a run_in_group command ended, from how run_formatter_child saw its child end or its
//! io error.

use crate::child_exit::ChildExit;
use crate::formatter_stderr_cap::FORMATTER_STDERR_CAP;
use crate::group_run_end::GroupRunEnd;
use std::io;

/// An exit, whatever its code (0 included), is Exited with its code and stderr. A child killed at its limit is
/// TimedOut with the limit and the stderr read before the kill. An io error (spawn, wait, or a joined stderr read) is
/// SpawnFailed with its text. Each stderr is cut to FORMATTER_STDERR_CAP bytes and read lossy as UTF-8.
pub(crate) fn decide_group_run_end(exit: io::Result<ChildExit>) -> GroupRunEnd {
    match exit {
        Ok(ChildExit::Exited { code, stderr }) => GroupRunEnd::Exited { code, stderr: capped_text(&stderr) },
        Ok(ChildExit::TimedOut { limit, stderr }) => GroupRunEnd::TimedOut { limit, stderr: capped_text(&stderr) },
        Err(e) => GroupRunEnd::SpawnFailed { error: e.to_string() },
    }
}

/// The first FORMATTER_STDERR_CAP bytes of `stderr`, read lossy as UTF-8 (the cut may split a character; it shows as
/// a replacement character). The cut repeats read_capped's cap, so it holds for any ChildExit, however it was made.
fn capped_text(stderr: &[u8]) -> String {
    String::from_utf8_lossy(&stderr[..stderr.len().min(FORMATTER_STDERR_CAP)]).into_owned()
}
