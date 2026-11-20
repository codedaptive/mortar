//! open_requests_for: the zone's request folders whose branch exists, tipped at the snapshot when there is one,
//! based at the fork point of that tip and never at the record's base, in the order given; every
//! other folder is skipped without asking for a fork point.

use std::collections::BTreeMap;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;
use ::join::OpenRequest;

use super::open_requests_for;

const RECORD_BASE: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";
const SNAP: &str = "3333333333333333333333333333333333333333";
const TIP_FORK: &str = "5555555555555555555555555555555555555555";
const SNAP_FORK: &str = "6666666666666666666666666666666666666666";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(kind: FolderKind, zone: &str, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: zone.to_string(),
        request: request.map(str::to_string),
        base: oid(RECORD_BASE),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: None,
    }
}

fn open(name: &str, base: &str, tip: &str) -> OpenRequest {
    OpenRequest { name: name.to_string(), base: oid(base), tip: oid(tip) }
}

/// A fake fork point: TIP forks at TIP_FORK and SNAP at SNAP_FORK; any other tip is an error naming it. Every call
/// is logged as (name, tip).
fn fork_table(calls: &mut Vec<(String, Oid)>) -> impl FnMut(&str, &Oid) -> Result<Oid, String> + '_ {
    let table = BTreeMap::from([(TIP, TIP_FORK), (SNAP, SNAP_FORK)]);
    move |name, tip| {
        calls.push((name.to_string(), tip.clone()));
        table.get(tip.as_str()).map(|base| oid(base)).ok_or(format!("no fork point for {name}"))
    }
}

#[test]
fn the_base_is_the_fork_point_of_the_snapshot_or_else_of_the_branch_tip_never_the_record_base() {
    let folders = [
        (record(FolderKind::Request, "core", Some("fix-1")), Some(oid(TIP)), Some(oid(SNAP))),
        (record(FolderKind::Request, "core", Some("fix-2")), Some(oid(TIP)), None),
    ];
    let mut calls = Vec::new();
    let open_requests = open_requests_for("core", &folders, fork_table(&mut calls));
    assert_eq!(open_requests, Ok(vec![open("fix-1", SNAP_FORK, SNAP), open("fix-2", TIP_FORK, TIP)]));
    assert_eq!(calls, vec![("fix-1".to_string(), oid(SNAP)), ("fix-2".to_string(), oid(TIP))]);
}

#[test]
fn zone_attempt_other_zone_and_branchless_folders_are_skipped_without_a_fork_point() {
    let folders = [
        (record(FolderKind::Zone, "core", None), Some(oid(TIP)), None),
        (record(FolderKind::Attempt(1), "core", Some("fix-1")), Some(oid(TIP)), None),
        (record(FolderKind::Request, "web", Some("fix-3")), Some(oid(TIP)), None),
        (record(FolderKind::Request, "core", Some("gone")), None, Some(oid(SNAP))),
        (record(FolderKind::Request, "core", Some("kept")), Some(oid(TIP)), None),
    ];
    let mut calls = Vec::new();
    let open_requests = open_requests_for("core", &folders, fork_table(&mut calls));
    assert_eq!(open_requests, Ok(vec![open("kept", TIP_FORK, TIP)]));
    assert_eq!(calls, vec![("kept".to_string(), oid(TIP))]);
}

#[test]
fn the_first_fork_point_error_is_returned_and_stops_the_reading() {
    let unknown = "7777777777777777777777777777777777777777";
    let folders = [
        (record(FolderKind::Request, "core", Some("lost")), Some(oid(unknown)), None),
        (record(FolderKind::Request, "core", Some("fine")), Some(oid(TIP)), None),
    ];
    let mut calls = Vec::new();
    let open_requests = open_requests_for("core", &folders, fork_table(&mut calls));
    assert_eq!(open_requests, Err("no fork point for lost".to_string()));
    assert_eq!(calls, vec![("lost".to_string(), oid(unknown))]);
}

#[test]
fn no_folders_is_no_open_request_and_no_fork_point() {
    let mut calls = Vec::new();
    assert_eq!(open_requests_for("core", &[], fork_table(&mut calls)), Ok(Vec::new()));
    assert_eq!(calls, Vec::new());
}
