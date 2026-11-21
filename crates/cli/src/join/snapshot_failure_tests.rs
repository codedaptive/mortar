//! snapshot_failure: an unborn or detached HEAD is a refusal owned by the folder's HEAD, and every other snapshot
//! error is a failure, for the verb passed, naming the error in its Display form.
//! Unit tests: the error is built as data; its line comes from snapshot's exported, pure Display, and
//! report::failure_report is used directly.

use std::path::PathBuf;

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
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn an_unborn_head_for_undo_is_refused_naming_the_head() {
    let error = SnapshotError::UnbornHead { folder: PathBuf::from("/w/a") };
    assert_eq!(
        snapshot_failure("undo", error),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["HEAD of the folder /w/a has no commit".to_string()],
            owner: Some("the folder's HEAD".to_string()),
            next: "check out the request's branch in the folder, then repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_detached_head_for_folder_keep_is_refused_naming_the_head() {
    let error = SnapshotError::DetachedHead { folder: PathBuf::from("/w/a") };
    assert_eq!(
        snapshot_failure("folder keep", error),
        Report {
            verb: "folder keep".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["HEAD of the folder /w/a names no branch".to_string()],
            owner: Some("the folder's HEAD".to_string()),
            next: "check out the request's branch in the folder, then repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
