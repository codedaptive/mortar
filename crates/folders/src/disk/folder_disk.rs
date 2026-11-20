//! One folder's line in the disk report.
//! A bare type; sweep_figure_tests builds it.

use std::path::PathBuf;

use crate::record::FolderRecord;

/// A Mortar folder at `path` with its `record`: `files_bytes` is everything in it but `.git` and the build dir,
/// `build_bytes` the build dir alone (both apparent sizes), and `newest` the newest modification time over both,
/// in unix seconds (None when nothing was found).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderDisk {
    pub path: PathBuf,
    pub record: FolderRecord,
    pub files_bytes: u64,
    pub build_bytes: u64,
    pub newest: Option<u64>,
}
