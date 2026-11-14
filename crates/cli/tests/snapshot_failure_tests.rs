//! snapshot_failure: a snapshot error is a failure for the verb passed, naming the error in its Display form.
//! Integration group. A ZIT (S-79 as replaced: the line is the file): it calls report::failure_report for real.
//! By S-101 also a RIT case, since it runs snapshot's Display for real; listed in
//! crates/cli/src/join/snapshot_failure.rs's header.

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
