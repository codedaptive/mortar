//! folder_kind_text: each folder kind in the cli's words, with its zone and request.

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::folder_kind_text;

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "core".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: None,
    }
}

#[test]
fn a_zone_folder_names_its_zone() {
    assert_eq!(folder_kind_text(&record(FolderKind::Zone, None)), "the zone folder of zone core");
}

#[test]
fn a_request_folder_names_its_request_and_zone() {
    assert_eq!(
        folder_kind_text(&record(FolderKind::Request, Some("r1"))),
        "the request folder of request r1 in zone core"
    );
}

#[test]
fn an_attempt_folder_names_its_number_request_and_zone() {
    assert_eq!(
        folder_kind_text(&record(FolderKind::Attempt(2), Some("r1"))),
        "attempt folder 2 of request r1 in zone core"
    );
}
