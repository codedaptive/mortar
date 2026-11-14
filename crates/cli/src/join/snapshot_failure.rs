//! The report of a snapshot or a save that failed inside a request verb.
//! Nagatha's RIT (S-101), a_bad_target_is_a_failure_naming_it in crates/cli/tests/snapshot_failure_tests.rs,
//! joins this CLI report to snapshot::SnapshotError's Display. It checks the passed verb, exit 5/outcome "failed",
//! the error text "the snapshot ref cannot be named: attempt 0", the second line "nothing half-done is left",
//! and next "repeat the same command".
//! No sibling snapshot_failure_tests.rs (S-104): its one test runs snapshot's Display for real.

use snapshot::SnapshotError;

use crate::report::{failure_report, Report};

/// failure_report for `verb` with the error's Display text.
pub fn snapshot_failure(verb: &str, error: SnapshotError) -> Report {
    failure_report(verb, &error.to_string())
}
