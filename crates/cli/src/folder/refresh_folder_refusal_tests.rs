//! refresh_folder_refusal: a zone folder is not refused; a request or attempt folder is, naming what it is, with
//! mr join in that folder as next.

use std::path::Path;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::refresh_folder_refusal;
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

fn refused(first: &str) -> Option<Report> {
    Some(Report {
        verb: "folder".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![first.to_string(), "nothing changed".to_string()],
        owner: Some("the harness that makes Mortar folders".to_string()),
        next: "run mr join in /w/f to bring it up to its zone".to_string(),
        fields: Vec::new(),
    })
}

#[test]
fn only_a_zone_folder_is_not_refused() {
    let rows: Vec<(&str, FolderRecord, Option<Report>)> = vec![
        ("zone", record(FolderKind::Zone, None), None),
        (
            "request",
            record(FolderKind::Request, Some("fix-1")),
            refused("/w/f is the request folder of request fix-1 in zone core: mr folder refresh runs on a zone folder"),
        ),
        (
            "attempt",
            record(FolderKind::Attempt(2), Some("fix-1")),
            refused("/w/f is attempt folder 2 of request fix-1 in zone core: mr folder refresh runs on a zone folder"),
        ),
        (
            "attempt naming no request",
            record(FolderKind::Attempt(1), None),
            refused("/w/f is attempt folder 1 of request (none) in zone core: mr folder refresh runs on a zone folder"),
        ),
    ];
    for (name, record, expected) in rows {
        assert_eq!(refresh_folder_refusal(Path::new("/w/f"), &record), expected, "row {name}");
    }
}
