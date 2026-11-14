use crate::snapshot_ref_name::snapshot_ref_name;
use crate::{SnapshotError, SnapshotTarget};

fn target(zone: &str, request: &str) -> SnapshotTarget {
    SnapshotTarget { zone: zone.to_string(), request: request.to_string(), actor: "bilby".to_string() }
}

/// The BadTarget reason, panicking on any other result.
fn refusal(zone: &str, request: &str) -> String {
    match snapshot_ref_name(&target(zone, request)) {
        Err(SnapshotError::BadTarget { reason }) => reason,
        other => panic!("expected BadTarget, got {other:?}"),
    }
}

#[test]
fn names_the_zone_and_request_ref() {
    assert_eq!(snapshot_ref_name(&target("snapshot", "WO-snapshot-1")).unwrap(), "refs/mortar/snap/snapshot/WO-snapshot-1");
}

#[test]
fn refuses_an_empty_zone_naming_it() {
    assert!(refusal("", "WO-snapshot-1").contains("zone"));
}

#[test]
fn refuses_a_zone_with_a_slash_naming_it_and_its_text() {
    let reason = refusal("snap/shot", "WO-snapshot-1");
    assert!(reason.contains("zone"));
    assert!(reason.contains("snap/shot"));
}

#[test]
fn refuses_an_empty_request_naming_it() {
    assert!(refusal("snapshot", "").contains("request"));
}

#[test]
fn refuses_a_request_with_a_slash_naming_it_and_its_text() {
    let reason = refusal("snapshot", "WO/1");
    assert!(reason.contains("request"));
    assert!(reason.contains("WO/1"));
}
