//! gaps: the needed paths the work order does not name, in needed_paths order.

use gitio::Oid;

use super::gaps::gaps;
use crate::record::{FolderKind, FolderRecord};

fn record(work_order_paths: &[&str], needed_paths: &[&str]) -> FolderRecord {
    FolderRecord {
        kind: FolderKind::Zone,
        zone: "folders".to_string(),
        request: None,
        base: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        work_order_paths: work_order_paths.iter().map(|p| p.to_string()).collect(),
        needed_paths: needed_paths.iter().map(|p| p.to_string()).collect(),
        parent: None,
    }
}

#[test]
fn nothing_needed_has_no_gaps() {
    assert_eq!(gaps(&record(&["crates/folders"], &[])), Vec::<String>::new());
}

#[test]
fn needed_paths_the_work_order_covers_are_no_gaps() {
    assert_eq!(gaps(&record(&["crates/folders", "Cargo.toml"], &["Cargo.toml"])), Vec::<String>::new());
}

#[test]
fn only_the_needed_paths_outside_the_work_order_are_gaps() {
    let found = gaps(&record(&["crates/folders"], &["crates/folders", "crates/gitio"]));
    assert_eq!(found, vec!["crates/gitio".to_string()]);
}

#[test]
fn gaps_keep_needed_paths_order() {
    let found = gaps(&record(&["crates/folders"], &["crates/oplog", "crates/folders", "Cargo.toml", "crates/gitio"]));
    assert_eq!(
        found,
        vec!["crates/oplog".to_string(), "Cargo.toml".to_string(), "crates/gitio".to_string()]
    );
}
