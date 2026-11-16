//! folder_removed_report: a request folder removed, its request and zone named, and nothing more to do.

use std::path::Path;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::folder_removed_report;
use crate::report::{ExitCode, OutValue, Report};

#[test]
fn a_request_folder_removed() {
    let record = FolderRecord {
        kind: FolderKind::Request,
        zone: "core".to_string(),
        request: Some("fix-1".to_string()),
        base: Oid::parse(&"e".repeat(40)).expect("a valid oid"),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: Some("zone/core".to_string()),
    };
    assert_eq!(
        folder_removed_report(Path::new("/w/fix"), &record),
        Report {
            verb: "folder".to_string(),
            exit: ExitCode::Done,
            outcome: "removed".to_string(),
            lines: vec![
                "removed the request folder of request fix-1 in zone core at /w/fix, with its branch and its \
                 snapshot ref"
                    .to_string()
            ],
            owner: None,
            next: "nothing more for this request".to_string(),
            fields: vec![
                ("path".to_string(), OutValue::Text("/w/fix".to_string())),
                ("zone".to_string(), OutValue::Text("core".to_string())),
                ("request".to_string(), OutValue::Text("fix-1".to_string())),
            ],
        }
    );
}
