//! refill_cone: the record's paths as one sorted, deduplicated cone; no paths is refused.

use gitio::Oid;

use super::refill_cone::refill_cone;
use super::FolderError;
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
fn work_order_paths_alone() {
    let cone = refill_cone(&record(&["crates/folders", "Cargo.toml"], &[])).unwrap();
    assert_eq!(cone, vec!["Cargo.toml".to_string(), "crates/folders".to_string()]);
}

#[test]
fn work_order_and_needed_paths_are_one_sorted_deduplicated_list() {
    let cone = refill_cone(&record(&["crates/oplog", "crates/folders"], &["crates/gitio", "crates/folders"])).unwrap();
    assert_eq!(
        cone,
        vec![
            "crates/folders".to_string(),
            "crates/gitio".to_string(),
            "crates/oplog".to_string()
        ]
    );
}

#[test]
fn a_record_with_no_paths_is_refused() {
    let err = refill_cone(&record(&[], &[])).unwrap_err();
    match err {
        FolderError::Plan(plan) => assert_eq!(plan.reason, "no sparse paths given: a folder needs at least one"),
        other => panic!("expected Plan, got {other:?}"),
    }
}
