//! sweep_figure: the build bytes of the idle folders alone, and the reason naming the folder count, the days and
//! the closed snapshot refs.

use std::path::PathBuf;

use gitio::Oid;

use super::sweep_figure::sweep_figure;
use super::FolderDisk;
use crate::plan::SweepEstimate;
use crate::record::{FolderKind, FolderRecord};

fn folder(path: &str, files_bytes: u64, build_bytes: u64) -> FolderDisk {
    FolderDisk {
        path: PathBuf::from(path),
        record: FolderRecord {
            kind: FolderKind::Request,
            zone: "folders".to_string(),
            request: Some("WO-1".to_string()),
            base: Oid::parse("0123456789abcdef0123456789abcdef01234567").unwrap(),
            work_order_paths: Vec::new(),
            needed_paths: Vec::new(),
            parent: None,
        },
        files_bytes,
        build_bytes,
        newest: None,
    }
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

#[test]
fn sums_the_build_bytes_of_idle_folders_only() {
    let folders = [folder("/w/a", 1_000, 300), folder("/w/b", 5, 4_000), folder("/w/c", 9, 20)];
    assert_eq!(
        sweep_figure(&folders, &paths(&["/w/a", "/w/c"]), 3, 14),
        SweepEstimate {
            bytes: 320,
            reason: Some(
                "build dirs of 2 folders idle 14 days or more; 3 snapshot refs of closed requests, their objects \
                 not measured"
                    .to_string()
            ),
        }
    );
}

#[test]
fn an_idle_path_with_no_folder_counts_nothing() {
    let folders = [folder("/w/a", 1, 50)];
    assert_eq!(
        sweep_figure(&folders, &paths(&["/w/gone"]), 0, 7),
        SweepEstimate {
            bytes: 0,
            reason: Some(
                "build dirs of 0 folders idle 7 days or more; 0 snapshot refs of closed requests, their objects not \
                 measured"
                    .to_string()
            ),
        }
    );
}

#[test]
fn no_folders_free_nothing() {
    assert_eq!(sweep_figure(&[], &[], 1, 30).bytes, 0);
}

#[test]
fn the_sum_saturates() {
    let folders = [folder("/w/a", 0, u64::MAX), folder("/w/b", 0, 2)];
    assert_eq!(sweep_figure(&folders, &paths(&["/w/a", "/w/b"]), 0, 1).bytes, u64::MAX);
}
