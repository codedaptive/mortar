//! holds_line: the folder's two counts apart, then joined, not joined, no joined answer for a zone folder, or joined
//! unknown with its reason.

use std::path::Path;

use folders::{FolderKind, FolderRecord, UnsavedCounts};
use gitio::Oid;

use super::{holds_line, FolderHolds, JoinedAnswer};

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

fn holds(tracked: u64, untracked: u64, joined: JoinedAnswer) -> FolderHolds {
    FolderHolds { unsaved: UnsavedCounts { tracked, untracked }, joined }
}

#[test]
fn a_joined_request_says_joined_to_its_zone() {
    assert_eq!(
        holds_line(
            Path::new("/w/fix-1"),
            &record(FolderKind::Request, Some("fix-1")),
            &holds(0, 0, JoinedAnswer::Joined)
        ),
        "/w/fix-1: unsaved 0 tracked, 0 untracked; joined to zone core"
    );
}

#[test]
fn an_unjoined_request_says_not_joined_with_its_counts_apart() {
    assert_eq!(
        holds_line(
            Path::new("/w/fix-1"),
            &record(FolderKind::Request, Some("fix-1")),
            &holds(1, 2, JoinedAnswer::NotJoined)
        ),
        "/w/fix-1: unsaved 1 tracked, 2 untracked; not joined to zone core"
    );
}

#[test]
fn a_zone_folder_has_no_joined_answer() {
    assert_eq!(
        holds_line(Path::new("/w/core"), &record(FolderKind::Zone, None), &holds(3, 4, JoinedAnswer::ZoneFolder)),
        "/w/core: unsaved 3 tracked, 4 untracked; a zone folder has no joined answer"
    );
}

#[test]
fn a_failed_joined_read_says_joined_unknown_with_its_reason() {
    let unknown = JoinedAnswer::Unknown("the branch refs/heads/req/core/fix-1 does not exist".to_string());
    assert_eq!(
        holds_line(Path::new("/w/fix-1"), &record(FolderKind::Request, Some("fix-1")), &holds(5, 6, unknown)),
        "/w/fix-1: unsaved 5 tracked, 6 untracked; joined unknown: the branch refs/heads/req/core/fix-1 does not exist"
    );
}

#[test]
fn no_line_says_safe() {
    let answers =
        [JoinedAnswer::Joined, JoinedAnswer::NotJoined, JoinedAnswer::ZoneFolder, JoinedAnswer::Unknown("x".into())];
    for joined in answers {
        let line = holds_line(Path::new("/w/x"), &record(FolderKind::Request, Some("x")), &holds(0, 0, joined));
        assert!(!line.to_lowercase().contains("safe"), "{line}");
    }
}
