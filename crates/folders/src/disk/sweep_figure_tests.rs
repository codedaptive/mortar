//! sweep_figure: the build bytes of the idle folders plus the ref sweep's bytes, an upper bound, and the reason
//! naming both parts with their counts.

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
fn sums_the_idle_folders_build_bytes_and_the_refs_bytes() {
    let folders = [folder("/w/a", 1_000, 300), folder("/w/b", 5, 4_000), folder("/w/c", 9, 20)];
    assert_eq!(
        sweep_figure(&folders, &paths(&["/w/a", "/w/c"]), 14, 3, 1_000),
        SweepEstimate {
            bytes: 1_320,
            reason: Some(
                "frees up to 1320 bytes: 320 bytes in the build dirs of 2 folders idle 14 days or more, and up to \
                 1000 bytes of objects reachable only from the 3 refs a ref sweep deletes"
                    .to_string()
            ),
        }
    );
}

#[test]
fn an_idle_path_with_no_folder_counts_nothing() {
    let folders = [folder("/w/a", 1, 50)];
    assert_eq!(
        sweep_figure(&folders, &paths(&["/w/gone"]), 7, 2, 64),
        SweepEstimate {
            bytes: 64,
            reason: Some(
                "frees up to 64 bytes: 0 bytes in the build dirs of 0 folders idle 7 days or more, and up to 64 \
                 bytes of objects reachable only from the 2 refs a ref sweep deletes"
                    .to_string()
            ),
        }
    );
}

#[test]
fn no_folders_and_no_refs_free_nothing() {
    assert_eq!(
        sweep_figure(&[], &[], 30, 0, 0),
        SweepEstimate {
            bytes: 0,
            reason: Some(
                "frees up to 0 bytes: 0 bytes in the build dirs of 0 folders idle 30 days or more, and up to 0 \
                 bytes of objects reachable only from the 0 refs a ref sweep deletes"
                    .to_string()
            ),
        }
    );
}

#[test]
fn the_build_sum_saturates() {
    let folders = [folder("/w/a", 0, u64::MAX), folder("/w/b", 0, 2)];
    assert_eq!(sweep_figure(&folders, &paths(&["/w/a", "/w/b"]), 1, 0, 0).bytes, 18_446_744_073_709_551_615);
}

#[test]
fn build_plus_refs_bytes_saturates() {
    let folders = [folder("/w/a", 0, 10)];
    assert_eq!(sweep_figure(&folders, &paths(&["/w/a"]), 1, 1, u64::MAX - 5).bytes, 18_446_744_073_709_551_615);
}
