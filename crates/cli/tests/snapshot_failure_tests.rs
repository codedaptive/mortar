//! snapshot_failure: a snapshot error is a failure for the verb passed, naming the error in its Debug form.
//! ZIT (S-79 as replaced: the line is the file): it calls report::failure_report for real.

use snapshot::SnapshotError;

use cli::{snapshot_failure, ExitCode, Report};

#[test]
fn a_bad_target_is_a_failure_naming_it() {
    let error = SnapshotError::BadTarget { reason: "attempt 0".to_string() };
    assert_eq!(
        snapshot_failure("resolve", error),
        Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec!["BadTarget { reason: \"attempt 0\" }".to_string(), "nothing half-done is left".to_string()],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
