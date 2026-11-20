//! disk_report: per folder, a line with its files and build bytes then a line with what it holds; the object store
//! counted once, the ref counts by namespace and what a plain mr sweep frees, in the cli's words (never folders'
//! reason text); the holds field last; the next is a sweep of idle build dirs when any build bytes exist, else
//! nothing to do.

use std::path::PathBuf;

use folders::{DiskReport, FolderDisk, FolderKind, FolderRecord, SweepEstimate, UnsavedCounts};
use gitio::Oid;

use super::{disk_report, FolderHolds};
use crate::report::{ExitCode, OutValue, Report};

const BASE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const REASON: &str = "folders reason text 8812";

fn folder(path: &str, kind: FolderKind, request: Option<&str>, files_bytes: u64, build_bytes: u64) -> FolderDisk {
    FolderDisk {
        path: PathBuf::from(path),
        record: FolderRecord {
            kind,
            zone: "core".to_string(),
            request: request.map(str::to_string),
            base: Oid::parse(BASE).expect("a valid oid"),
            work_order_paths: Vec::new(),
            needed_paths: Vec::new(),
            parent: None,
        },
        files_bytes,
        build_bytes,
        newest: Some(1_763_100_000),
    }
}

fn disk(request_build_bytes: u64) -> DiskReport {
    DiskReport {
        folders: vec![
            folder("/w/core", FolderKind::Zone, None, 1000, 0),
            folder("/w/fix-1", FolderKind::Request, Some("fix-1"), 300, request_build_bytes),
        ],
        store_bytes: 50_000,
        refs: vec![("refs/heads".to_string(), 12), ("refs/mortar".to_string(), 7)],
        sweep: SweepEstimate { bytes: 4096, reason: Some(REASON.to_string()) },
    }
}

/// The holds of disk()'s two folders: the zone folder with 3 tracked and 4 untracked, the request joined.
const HOLDS: [FolderHolds; 2] = [
    FolderHolds { unsaved: UnsavedCounts { tracked: 3, untracked: 4 }, joined: None },
    FolderHolds { unsaved: UnsavedCounts { tracked: 0, untracked: 0 }, joined: Some(true) },
];

#[test]
fn a_zone_and_a_request_folder_are_measured_with_the_store_counted_once() {
    assert_eq!(
        disk_report(&disk(2000), &HOLDS),
        Report {
            verb: "disk".to_string(),
            exit: ExitCode::Done,
            outcome: "measured".to_string(),
            lines: vec![
                "/w/core: the zone folder of zone core; files 1000 bytes, build 0 bytes".to_string(),
                "/w/core: unsaved 3 tracked, 4 untracked; a zone folder has no joined answer".to_string(),
                "/w/fix-1: the request folder of request fix-1 in zone core; files 300 bytes, build 2000 bytes"
                    .to_string(),
                "/w/fix-1: unsaved 0 tracked, 0 untracked; joined to zone core".to_string(),
                "object store: 50000 bytes, counted once".to_string(),
                "refs by namespace: refs/heads 12, refs/mortar 7".to_string(),
                "an mr sweep with no --idle-days frees up to 4096 bytes".to_string(),
            ],
            owner: None,
            next: "mr sweep --idle-days <n> --actor <name>".to_string(),
            fields: vec![
                ("folders".to_string(), OutValue::Texts(vec!["/w/core".to_string(), "/w/fix-1".to_string()])),
                ("store_bytes".to_string(), OutValue::Number(50_000)),
                ("refs".to_string(), OutValue::Texts(vec!["refs/heads 12".to_string(), "refs/mortar 7".to_string()])),
                ("sweep_bytes".to_string(), OutValue::Number(4096)),
                (
                    "holds".to_string(),
                    OutValue::Raw(vec![
                        r#"{"path":"/w/core","tracked":3,"untracked":4,"joined":null}"#.to_string(),
                        r#"{"path":"/w/fix-1","tracked":0,"untracked":0,"joined":true}"#.to_string(),
                    ]),
                ),
            ],
        }
    );
}

#[test]
fn an_attempt_folder_is_named_with_its_number() {
    let mut report = disk(0);
    report.folders = vec![folder("/w/tries/try-2", FolderKind::Attempt(2), Some("fix-1"), 10, 20)];
    assert_eq!(
        disk_report(&report, &HOLDS[1..]).lines[0],
        "/w/tries/try-2: attempt folder 2 of request fix-1 in zone core; files 10 bytes, build 20 bytes"
    );
}

#[test]
fn no_build_bytes_anywhere_leaves_nothing_to_do() {
    assert_eq!(disk_report(&disk(0), &HOLDS).next, "nothing to do");
}

#[test]
fn any_build_bytes_point_at_a_sweep_of_idle_build_dirs() {
    assert_eq!(disk_report(&disk(1), &HOLDS).next, "mr sweep --idle-days <n> --actor <name>");
}

#[test]
fn folders_reason_text_is_never_printed() {
    let printed = format!("{:?}", disk_report(&disk(2000), &HOLDS));
    assert!(!printed.contains(REASON), "{printed}");
}

#[test]
fn the_store_line_appears_once_whatever_the_number_of_folders() {
    let lines = disk_report(&disk(2000), &HOLDS).lines;
    assert_eq!(lines.iter().filter(|line| line.starts_with("object store: ")).count(), 1);
}

#[test]
fn an_unjoined_request_says_not_joined_under_its_bytes_line() {
    let held = [HOLDS[0], FolderHolds { unsaved: UnsavedCounts { tracked: 1, untracked: 2 }, joined: Some(false) }];
    let report = disk_report(&disk(0), &held);
    assert_eq!(report.lines[3], "/w/fix-1: unsaved 1 tracked, 2 untracked; not joined to zone core");
    assert_eq!(
        report.fields.last(),
        Some(&(
            "holds".to_string(),
            OutValue::Raw(vec![
                r#"{"path":"/w/core","tracked":3,"untracked":4,"joined":null}"#.to_string(),
                r#"{"path":"/w/fix-1","tracked":1,"untracked":2,"joined":false}"#.to_string(),
            ])
        ))
    );
}

#[test]
fn no_line_says_safe() {
    for joined in [Some(true), Some(false)] {
        let held = [HOLDS[0], FolderHolds { unsaved: UnsavedCounts::default(), joined }];
        let report = disk_report(&disk(2000), &held);
        assert!(report.lines.iter().all(|line| !line.to_lowercase().contains("safe")), "{:?}", report.lines);
    }
}
