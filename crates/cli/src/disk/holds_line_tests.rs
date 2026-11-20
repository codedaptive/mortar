//! holds_line: the folder's two counts apart, then joined, not joined, or no joined answer for a zone folder.

use std::path::Path;

use folders::{FolderKind, FolderRecord, UnsavedCounts};
use gitio::Oid;

use super::{holds_line, FolderHolds};

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "core".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: None,
    }
}

fn holds(tracked: u64, untracked: u64, joined: Option<bool>) -> FolderHolds {
    FolderHolds { unsaved: UnsavedCounts { tracked, untracked }, joined }
}

#[test]
fn a_joined_request_says_joined_to_its_zone() {
    assert_eq!(
        holds_line(Path::new("/w/fix-1"), &record(FolderKind::Request, Some("fix-1")), &holds(0, 0, Some(true))),
        "/w/fix-1: unsaved 0 tracked, 0 untracked; joined to zone core"
    );
}

#[test]
fn an_unjoined_request_says_not_joined_with_its_counts_apart() {
    assert_eq!(
        holds_line(Path::new("/w/fix-1"), &record(FolderKind::Request, Some("fix-1")), &holds(1, 2, Some(false))),
        "/w/fix-1: unsaved 1 tracked, 2 untracked; not joined to zone core"
    );
}

#[test]
fn a_zone_folder_has_no_joined_answer() {
    assert_eq!(
        holds_line(Path::new("/w/core"), &record(FolderKind::Zone, None), &holds(3, 4, None)),
        "/w/core: unsaved 3 tracked, 4 untracked; a zone folder has no joined answer"
    );
}

#[test]
fn no_line_says_safe() {
    for joined in [Some(true), Some(false), None] {
        let line = holds_line(Path::new("/w/x"), &record(FolderKind::Request, Some("x")), &holds(0, 0, joined));
        assert!(!line.to_lowercase().contains("safe"), "{line}");
    }
}
