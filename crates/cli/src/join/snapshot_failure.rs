//! The report of a snapshot or a save that failed inside a request verb.
//! snapshot_failure_tests covers it with snapshot::SnapshotError's Display, exported and pure (a ZUT by S-171).

use snapshot::SnapshotError;

use crate::report::{failure_report, Report};

/// failure_report for `verb` with the error's Display text.
pub fn snapshot_failure(verb: &str, error: SnapshotError) -> Report {
    failure_report(verb, &error.to_string())
}
