//! add_needed: a new path joins needed_paths in sorted order; a path already needed answers None.

use gitio::Oid;

use super::add_needed::add_needed;
use crate::record::{FolderKind, FolderRecord};

fn record(work_order_paths: &[&str], needed_paths: &[&str]) -> FolderRecord {
    FolderRecord {
        kind: FolderKind::Request,
        zone: "folders".to_string(),
        request: Some("rf2".to_string()),
        base: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        work_order_paths: work_order_paths.iter().map(|p| p.to_string()).collect(),
        needed_paths: needed_paths.iter().map(|p| p.to_string()).collect(),
        parent: Some("zone/folders".to_string()),
    }
}

fn strings(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| p.to_string()).collect()
}

#[test]
fn a_new_path_is_added_in_sorted_order() {
    let new = add_needed(&record(&["crates/folders"], &["Cargo.toml", "crates/oplog"]), "crates/gitio").unwrap();
    assert_eq!(new.needed_paths, strings(&["Cargo.toml", "crates/gitio", "crates/oplog"]));
}

#[test]
fn the_first_needed_path_is_added_to_an_empty_list() {
    let new = add_needed(&record(&["crates/folders"], &[]), "crates/gitio").unwrap();
    assert_eq!(new.needed_paths, strings(&["crates/gitio"]));
}

#[test]
fn a_path_already_needed_answers_none() {
    assert_eq!(add_needed(&record(&["crates/folders"], &["crates/gitio", "crates/oplog"]), "crates/oplog"), None);
}

#[test]
fn a_work_order_path_not_yet_needed_is_still_added() {
    let new = add_needed(&record(&["crates/folders"], &[]), "crates/folders").unwrap();
    assert_eq!(new.needed_paths, strings(&["crates/folders"]));
}

#[test]
fn every_other_field_is_unchanged() {
    let new = add_needed(&record(&["crates/folders"], &["crates/oplog"]), "crates/gitio").unwrap();
    assert_eq!(new.kind, FolderKind::Request);
    assert_eq!(new.zone, "folders");
    assert_eq!(new.request, Some("rf2".to_string()));
    assert_eq!(new.base, Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap());
    assert_eq!(new.work_order_paths, strings(&["crates/folders"]));
    assert_eq!(new.parent, Some("zone/folders".to_string()));
}
