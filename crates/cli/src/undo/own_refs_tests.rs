//! own_refs: a request folder's own refs are its branch and its snapshot ref, attempt folder k's its try branch and
//! its trysnap ref, as full ref names in that order; a zone folder has none.

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::own_refs;

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
fn a_request_folder_owns_its_branch_then_its_snapshot_ref() {
    assert_eq!(
        own_refs(&record(FolderKind::Request, Some("r1"))),
        Some(vec!["refs/heads/req/a/r1".to_string(), "refs/mortar/snap/a/r1".to_string()])
    );
}

#[test]
fn attempt_folder_two_owns_its_try_branch_then_its_trysnap_ref() {
    assert_eq!(
        own_refs(&record(FolderKind::Attempt(2), Some("r1"))),
        Some(vec!["refs/heads/try/a/r1/2".to_string(), "refs/mortar/trysnap/a/r1/2".to_string()])
    );
}

#[test]
fn a_zone_folder_owns_no_refs_undo_can_reach() {
    assert_eq!(own_refs(&record(FolderKind::Zone, None)), None);
}
