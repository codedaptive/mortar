//! snapshot_failure: a snapshot error is a failure for the verb passed, naming the error in its Display form.
//! ZUT (rule 11, S-171): the error is built as data; its line comes from snapshot's exported, pure Display, and
//! report::failure_report is used directly.

use snapshot::SnapshotError;

use super::snapshot_failure;
use crate::report::{ExitCode, Report};

#[test]
fn a_bad_target_is_a_failure_naming_it() {
    let error = SnapshotError::BadTarget { reason: "attempt 0".to_string() };
    assert_eq!(
        snapshot_failure("resolve", error),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "the snapshot ref cannot be named: attempt 0".to_string(),
                "nothing half-done is left".to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
