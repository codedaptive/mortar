//! snapshot_target_for: a request folder saves to the request's ref, attempt folder k to attempt k's, and a
//! record with no request gives no target.

use folders::{FolderKind, FolderRecord};
use gitio::Oid;
use snapshot::SnapshotTarget;

use super::snapshot_target_for;

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "a".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: Some("zone/a".to_string()),
    }
}

#[test]
fn a_request_folder_targets_the_request_with_no_attempt() {
    let expected = SnapshotTarget {
        zone: "a".to_string(),
        request: "r1".to_string(),
        attempt: None,
        actor: "a/r1".to_string(),
    };
    assert_eq!(
        snapshot_target_for(&record(FolderKind::Request, Some("r1")), "a/r1"),
        Some(expected)
    );
}

#[test]
fn attempt_folder_three_targets_attempt_three() {
    let expected = SnapshotTarget {
        zone: "a".to_string(),
        request: "r1".to_string(),
        attempt: Some(3),
        actor: "a/r1".to_string(),
    };
    assert_eq!(
        snapshot_target_for(&record(FolderKind::Attempt(3), Some("r1")), "a/r1"),
        Some(expected)
    );
}

#[test]
fn a_record_with_no_request_gives_no_target() {
    assert_eq!(snapshot_target_for(&record(FolderKind::Zone, None), "a"), None);
}
