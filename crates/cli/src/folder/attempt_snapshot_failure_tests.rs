//! attempt_snapshot_failure: an unborn HEAD in an attempt folder is the HEAD refusal saying the attempt's branch
//! (without refs/heads/) no longer exists, and a detached one the HEAD refusal alone, each with next "repeat the
//! same command"; every other snapshot error is snapshot_failure's failure.
//! Unit tests: the error is built as data; its line comes from snapshot's exported, pure Display.

use std::path::PathBuf;

use snapshot::SnapshotError;

use super::attempt_snapshot_failure;
use crate::report::{ExitCode, Report};

#[test]
fn attempt_snapshot_failure_reports_each_error() {
    let next = "repeat the same command";
    let rows = [
        (
            "unborn head",
            SnapshotError::UnbornHead { folder: PathBuf::from("/w/try-1") },
            Report {
                verb: "folder".to_string(),
                exit: ExitCode::Refused,
                outcome: "refused".to_string(),
                lines: vec![
                    "HEAD of the folder /w/try-1 has no commit".to_string(),
                    "the branch try/core/task/1 no longer exists".to_string(),
                ],
                owner: Some("the folder's HEAD".to_string()),
                next: next.to_string(),
                fields: Vec::new(),
            },
        ),
        (
            "detached head",
            SnapshotError::DetachedHead { folder: PathBuf::from("/w/try-1") },
            Report {
                verb: "folder".to_string(),
                exit: ExitCode::Refused,
                outcome: "refused".to_string(),
                lines: vec!["HEAD of the folder /w/try-1 names no branch".to_string()],
                owner: Some("the folder's HEAD".to_string()),
                next: next.to_string(),
                fields: Vec::new(),
            },
        ),
        (
            "bad target",
            SnapshotError::BadTarget { reason: "attempt 0".to_string() },
            Report {
                verb: "folder".to_string(),
                exit: ExitCode::Failure,
                outcome: "failed".to_string(),
                lines: vec!["the snapshot ref cannot be named: attempt 0".to_string()],
                owner: None,
                next: "repeat the same command".to_string(),
                fields: Vec::new(),
            },
        ),
    ];
    for (case, error, expected) in rows {
        assert_eq!(attempt_snapshot_failure("folder", "refs/heads/try/core/task/1", error), expected, "{case}");
    }
}
