//! attempt_refusal: attempt folder 3 is the S-91 refusal for join and for resolve; request and zone folders give
//! none.

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::attempt_refusal;
use crate::report::{ExitCode, Report};

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "core".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: Some("zone/core".to_string()),
    }
}

fn refused(verb: &str, first: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![first.to_string(), "nothing ran".to_string()],
        owner: Some("this request".to_string()),
        next: "keep or drop the attempts, then mortar join in the request's folder".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn join_in_attempt_folder_three_is_refused() {
    assert_eq!(
        attempt_refusal("join", &record(FolderKind::Attempt(3), Some("fix-1"))),
        Some(refused("join", "this is attempt folder 3 of request fix-1: mortar join runs in the request's own folder"))
    );
}

#[test]
fn resolve_in_attempt_folder_three_is_refused() {
    assert_eq!(
        attempt_refusal("resolve", &record(FolderKind::Attempt(3), Some("fix-1"))),
        Some(refused(
            "resolve",
            "this is attempt folder 3 of request fix-1: mortar resolve runs in the request's own folder"
        ))
    );
}

#[test]
fn a_request_folder_is_not_refused() {
    assert_eq!(attempt_refusal("join", &record(FolderKind::Request, Some("fix-1"))), None);
}

#[test]
fn a_zone_folder_is_not_refused() {
    assert_eq!(attempt_refusal("join", &record(FolderKind::Zone, None)), None);
}
