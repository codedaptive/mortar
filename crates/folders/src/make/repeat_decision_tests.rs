//! repeat_decision: Same when only the base or needed paths differ, Different when any other field does.

use gitio::Oid;

use super::{repeat_decision, Repeat};
use crate::record::{FolderKind, FolderRecord};

fn record() -> FolderRecord {
    FolderRecord {
        kind: FolderKind::Request,
        zone: "folders".to_string(),
        request: Some("WO-folders-1".to_string()),
        base: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        work_order_paths: vec!["crates/folders".to_string()],
        needed_paths: vec![],
        parent: Some("zone/folders".to_string()),
    }
}

#[test]
fn equal_records_are_the_same() {
    assert_eq!(repeat_decision(&record(), &record()), Repeat::Same);
}

#[test]
fn a_different_base_alone_is_the_same() {
    let moved = FolderRecord {
        base: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &moved), Repeat::Same);
}

#[test]
fn a_different_kind_is_different() {
    let other = FolderRecord {
        kind: FolderKind::Zone,
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &other), Repeat::Different);
}

#[test]
fn a_different_zone_is_different() {
    let other = FolderRecord {
        zone: "join".to_string(),
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &other), Repeat::Different);
}

#[test]
fn a_different_request_is_different() {
    let other = FolderRecord {
        request: Some("WO-folders-2".to_string()),
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &other), Repeat::Different);
    let none = FolderRecord {
        request: None,
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &none), Repeat::Different);
}

#[test]
fn different_work_order_paths_are_different() {
    let other = FolderRecord {
        work_order_paths: vec!["crates/folders".to_string(), "crates/gitio".to_string()],
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &other), Repeat::Different);
}

#[test]
fn different_needed_paths_are_the_same() {
    let widened = FolderRecord {
        needed_paths: vec!["crates/oplog".to_string()],
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &widened), Repeat::Same);
}

#[test]
fn a_different_base_and_needed_paths_together_are_the_same() {
    let moved_and_widened = FolderRecord {
        base: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
        needed_paths: vec!["crates/oplog".to_string(), "crates/gitio".to_string()],
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &moved_and_widened), Repeat::Same);
}

#[test]
fn a_different_parent_is_different() {
    let other = FolderRecord {
        parent: None,
        ..record()
    };
    assert_eq!(repeat_decision(&record(), &other), Repeat::Different);
}
