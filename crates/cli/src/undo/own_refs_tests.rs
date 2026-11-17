//! own_refs: a request folder's own refs are its branch and its snapshot ref, attempt folder k's its try branch and
//! its trysnap ref, as full ref names in that order; a zone folder has none (S-142 item 2).

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::own_refs;

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "orc".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: Some("zone/orc".to_string()),
    }
}

#[test]
fn a_request_folder_owns_its_branch_then_its_snapshot_ref() {
    assert_eq!(
        own_refs(&record(FolderKind::Request, Some("WO-cli-4"))),
        Some(vec!["refs/heads/req/orc/WO-cli-4".to_string(), "refs/mortar/snap/orc/WO-cli-4".to_string()])
    );
}

#[test]
fn attempt_folder_two_owns_its_try_branch_then_its_trysnap_ref() {
    assert_eq!(
        own_refs(&record(FolderKind::Attempt(2), Some("WO-cli-4"))),
        Some(vec!["refs/heads/try/orc/WO-cli-4/2".to_string(), "refs/mortar/trysnap/orc/WO-cli-4/2".to_string()])
    );
}

#[test]
fn a_zone_folder_owns_no_refs_undo_can_reach() {
    assert_eq!(own_refs(&record(FolderKind::Zone, None)), None);
}
