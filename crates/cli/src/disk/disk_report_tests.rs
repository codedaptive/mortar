//! disk_report: one line per folder with its files and build bytes, the object store counted once, the ref counts by
//! namespace and what a plain mortar sweep frees, in the cli's words (never folders' reason text); the next is a sweep
//! of idle build dirs when any build bytes exist, else nothing to do.

use std::path::PathBuf;

use folders::{DiskReport, FolderDisk, FolderKind, FolderRecord, SweepEstimate};
use gitio::Oid;

use super::disk_report;
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

#[test]
fn a_zone_and_a_request_folder_are_measured_with_the_store_counted_once() {
    assert_eq!(
        disk_report(&disk(2000)),
        Report {
            verb: "disk".to_string(),
            exit: ExitCode::Done,
            outcome: "measured".to_string(),
            lines: vec![
                "/w/core: the zone folder of zone core; files 1000 bytes, build 0 bytes".to_string(),
                "/w/fix-1: the request folder of request fix-1 in zone core; files 300 bytes, build 2000 bytes"
                    .to_string(),
                "object store: 50000 bytes, counted once".to_string(),
                "refs by namespace: refs/heads 12, refs/mortar 7".to_string(),
                "a mortar sweep with no --idle-days frees up to 4096 bytes".to_string(),
            ],
            owner: None,
            next: "mortar sweep --idle-days <n> --actor <name>".to_string(),
            fields: vec![
                ("folders".to_string(), OutValue::Texts(vec!["/w/core".to_string(), "/w/fix-1".to_string()])),
                ("store_bytes".to_string(), OutValue::Number(50_000)),
                ("refs".to_string(), OutValue::Texts(vec!["refs/heads 12".to_string(), "refs/mortar 7".to_string()])),
                ("sweep_bytes".to_string(), OutValue::Number(4096)),
            ],
        }
    );
}

#[test]
fn an_attempt_folder_is_named_with_its_number() {
    let mut report = disk(0);
    report.folders = vec![folder("/w/tries/try-2", FolderKind::Attempt(2), Some("fix-1"), 10, 20)];
    assert_eq!(
        disk_report(&report).lines[0],
        "/w/tries/try-2: attempt folder 2 of request fix-1 in zone core; files 10 bytes, build 20 bytes"
    );
}

#[test]
fn no_build_bytes_anywhere_leaves_nothing_to_do() {
    assert_eq!(disk_report(&disk(0)).next, "nothing to do");
}

#[test]
fn any_build_bytes_point_at_a_sweep_of_idle_build_dirs() {
    assert_eq!(disk_report(&disk(1)).next, "mortar sweep --idle-days <n> --actor <name>");
}

#[test]
fn folders_reason_text_is_never_printed() {
    let printed = format!("{:?}", disk_report(&disk(2000)));
    assert!(!printed.contains(REASON), "{printed}");
}

#[test]
fn the_store_line_appears_once_whatever_the_number_of_folders() {
    let lines = disk_report(&disk(2000)).lines;
    assert_eq!(lines.iter().filter(|line| line.starts_with("object store: ")).count(), 1);
}
