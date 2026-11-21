//! The report of a snapshot or a save that failed or was refused inside a request verb.
//! snapshot_failure_tests covers it with snapshot::SnapshotError's Display, exported and pure
//! (so a unit test may use it).

use snapshot::SnapshotError;

use crate::report::{failure_report, repeat_head_refusal, Report};

/// An unborn or detached HEAD is repeat_head_refusal's refusal for `verb`, its one line the error's Display; every
/// other error is failure_report for `verb` with the error's Display text.
pub fn snapshot_failure(verb: &str, error: SnapshotError) -> Report {
    match error {
        SnapshotError::UnbornHead { .. } | SnapshotError::DetachedHead { .. } => {
            repeat_head_refusal(verb, vec![error.to_string()])
        }
        _ => failure_report(verb, &error.to_string()),
    }
}
