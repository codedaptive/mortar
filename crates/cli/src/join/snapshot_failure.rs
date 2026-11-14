//! The report of a snapshot or a save that failed inside a request verb.
//! crates/cli/tests/snapshot_failure_tests.rs covers it (a ZIT: it calls report::failure_report for real).

use snapshot::SnapshotError;

use crate::report::{failure_report, Report};

/// failure_report for `verb` with the error's Debug form: SnapshotError has no Display.
pub fn snapshot_failure(verb: &str, error: SnapshotError) -> Report {
    failure_report(verb, &format!("{error:?}"))
}
