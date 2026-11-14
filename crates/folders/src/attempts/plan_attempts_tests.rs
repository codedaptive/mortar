//! plan_attempts: attempts 1..=n in order with their branches and records; n == 0 and bad names refused.

use gitio::Oid;

use super::{plan_attempts, PlannedAttempt};
use crate::record::{FolderKind, FolderRecord};

const BASE: &str = "0123456789abcdef0123456789abcdef01234567";

fn base() -> Oid {
    Oid::parse(BASE).unwrap()
}

fn record(k: u32, cone: &[&str]) -> FolderRecord {
    FolderRecord {
        kind: FolderKind::Attempt(k),
        zone: "folders".to_string(),
        request: Some("WO-folders-6".to_string()),
        base: base(),
        work_order_paths: cone.iter().map(|p| p.to_string()).collect(),
        needed_paths: Vec::new(),
        parent: Some("req/folders/WO-folders-6".to_string()),
    }
}

#[test]
fn plans_n_attempts_in_order() {
    let cone = vec!["crates/folders".to_string(), "Cargo.toml".to_string()];
    let planned = plan_attempts("folders", "WO-folders-6", &base(), &cone, 3).unwrap();
    let wanted = ["crates/folders", "Cargo.toml"];
    assert_eq!(
        planned,
        vec![
            PlannedAttempt {
                k: 1,
                branch: "try/folders/WO-folders-6/1".to_string(),
                record: record(1, &wanted),
            },
            PlannedAttempt {
                k: 2,
                branch: "try/folders/WO-folders-6/2".to_string(),
                record: record(2, &wanted),
            },
            PlannedAttempt {
                k: 3,
                branch: "try/folders/WO-folders-6/3".to_string(),
                record: record(3, &wanted),
            },
        ]
    );
}

#[test]
fn plans_one_attempt_with_an_empty_cone() {
    let planned = plan_attempts("folders", "WO-folders-6", &base(), &[], 1).unwrap();
    assert_eq!(
        planned,
        vec![PlannedAttempt {
            k: 1,
            branch: "try/folders/WO-folders-6/1".to_string(),
            record: record(1, &[]),
        }]
    );
}

#[test]
fn refuses_a_count_of_zero() {
    assert_eq!(
        plan_attempts("folders", "WO-folders-6", &base(), &[], 0).unwrap_err().reason,
        "attempt count 0; make at least one"
    );
}

#[test]
fn refuses_a_bad_zone_or_request() {
    assert_eq!(
        plan_attempts(".z", "WO-1", &base(), &[], 2).unwrap_err().reason,
        "zone name \".z\" starts with '.' or '-'"
    );
    assert_eq!(
        plan_attempts("folders", "a..b", &base(), &[], 2).unwrap_err().reason,
        "request name \"a..b\" holds \"..\""
    );
}
