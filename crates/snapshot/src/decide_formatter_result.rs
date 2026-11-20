//! decide_formatter_result: how a formatter run ended, from how run_in_group says its command ended.

use crate::formatter_result::FormatterResult;
use crate::group_run_end::GroupRunEnd;

/// Exit 0 is Formatted, its stderr dropped (a formatter may warn and still succeed). Any other exit, or none (killed
/// by a signal), is Failed with its code and stderr. A run killed at its timeout is TimedOut with the limit and the
/// stderr read before the kill, whatever it would have exited with. SpawnFailed stays SpawnFailed with its text.
/// Failed, TimedOut and SpawnFailed are listed failures: the snapshot goes on.
pub(crate) fn decide_formatter_result(end: GroupRunEnd) -> FormatterResult {
    match end {
        GroupRunEnd::Exited { code: Some(0), .. } => FormatterResult::Formatted,
        GroupRunEnd::Exited { code, stderr } => FormatterResult::Failed { code, stderr },
        GroupRunEnd::TimedOut { limit, stderr } => FormatterResult::TimedOut { limit, stderr },
        GroupRunEnd::SpawnFailed { error } => FormatterResult::SpawnFailed { error },
    }
}
