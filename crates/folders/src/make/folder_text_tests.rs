//! The phrase folder_text gives for each kind of folder record, with and without a request.

use gitio::Oid;

use super::folder_text::folder_text;
use crate::record::{FolderKind, FolderRecord};

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "folders".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        work_order_paths: vec!["crates/folders".to_string()],
        needed_paths: vec![],
        parent: None,
    }
}

#[test]
fn zone_record_reads_as_the_zone_folder() {
    assert_eq!(folder_text(&record(FolderKind::Zone, None)), "the zone folder for zone folders");
}

#[test]
fn request_record_with_a_request_names_it() {
    assert_eq!(
        folder_text(&record(FolderKind::Request, Some("rf6"))),
        "the request folder for zone folders, request rf6"
    );
}

#[test]
fn request_record_with_no_request_reads_as_a_request_folder() {
    assert_eq!(folder_text(&record(FolderKind::Request, None)), "a request folder for zone folders");
}

#[test]
fn attempt_record_with_a_request_names_attempt_and_request() {
    assert_eq!(
        folder_text(&record(FolderKind::Attempt(2), Some("rf6"))),
        "attempt 2's folder for zone folders, request rf6"
    );
}

#[test]
fn attempt_record_with_no_request_names_the_attempt() {
    assert_eq!(folder_text(&record(FolderKind::Attempt(3), None)), "attempt 3's folder for zone folders");
}
