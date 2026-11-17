//! request_folder_refusal: a zone or attempt folder refuses the request-folder sub-verbs naming its kind; a request
//! folder does not.

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::request_folder_refusal;
use crate::report::{ExitCode, Report};

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "core".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(&"a".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: None,
    }
}

fn refused(first: &str, next: &str) -> Option<Report> {
    Some(Report {
        verb: "folder".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![first.to_string(), "nothing ran".to_string()],
        owner: Some("the harness that makes Mortar folders".to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    })
}

#[test]
fn a_request_folder_is_not_refused() {
    assert_eq!(request_folder_refusal("folder keep", &record(FolderKind::Request, Some("fix-1"))), None);
}

#[test]
fn a_zone_folder_is_refused_naming_its_zone() {
    assert_eq!(
        request_folder_refusal("folder attempts", &record(FolderKind::Zone, None)),
        refused(
            "this is the zone folder of zone core: mr folder attempts runs in a request folder",
            "run mr folder attempts --actor <name> in the request's own folder"
        )
    );
}

#[test]
fn an_attempt_folder_is_refused_naming_k_and_its_request() {
    assert_eq!(
        request_folder_refusal("folder keep", &record(FolderKind::Attempt(2), Some("fix-1"))),
        refused(
            "this is attempt folder 2 of request fix-1 in zone core: mr folder keep runs in a request folder",
            "run mr folder keep --actor <name> in the request's own folder"
        )
    );
}

#[test]
fn an_attempt_record_naming_no_request_is_still_refused() {
    assert_eq!(
        request_folder_refusal("folder drop", &record(FolderKind::Attempt(1), None)),
        refused(
            "this is attempt folder 1 in zone core: mr folder drop runs in a request folder",
            "run mr folder drop --actor <name> in the request's own folder"
        )
    );
}
