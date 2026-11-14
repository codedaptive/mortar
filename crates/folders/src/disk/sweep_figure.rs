//! What a sweep would free, from the disk report's folders.

use std::path::PathBuf;

use super::FolderDisk;
use crate::plan::SweepEstimate;

/// bytes: the sum of build_bytes over the folders whose path is in `idle` (saturating). reason: how many folders
/// that is, the `idle_days` it used, and the `closed_refs` snapshot refs a ref sweep would delete, whose objects
/// are not measured (they may be shared with live history).
pub fn sweep_figure(folders: &[FolderDisk], idle: &[PathBuf], closed_refs: usize, idle_days: u64) -> SweepEstimate {
    let swept: Vec<&FolderDisk> = folders.iter().filter(|f| idle.contains(&f.path)).collect();
    SweepEstimate {
        bytes: swept.iter().fold(0u64, |sum, f| sum.saturating_add(f.build_bytes)),
        reason: Some(format!(
            "build dirs of {} folders idle {idle_days} days or more; {closed_refs} snapshot refs of closed \
             requests, their objects not measured",
            swept.len()
        )),
    }
}
