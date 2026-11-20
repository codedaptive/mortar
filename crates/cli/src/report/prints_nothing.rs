//! prints_nothing: which reports the binary does not print at all.

use super::{ExitCode, Report};

/// True only for guard-hook's exit Done: guard-hook's only exit 0 is the guard allowing a move, and an allowed
/// move is silent on stdout and stderr, in both forms. git shows a hook's output on every ref transaction
/// and no caller reads an allow's report, so printing it would only add noise to every commit and push.
pub fn prints_nothing(report: &Report) -> bool {
    report.verb == "guard-hook" && report.exit == ExitCode::Done
}
