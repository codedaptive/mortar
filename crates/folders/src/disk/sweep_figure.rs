//! What a sweep would free, from the disk report's folders and the bytes of the refs a ref sweep deletes.

use std::path::PathBuf;

use super::FolderDisk;
use crate::plan::SweepEstimate;

/// bytes: the build_bytes of the folders whose path is in `idle`, plus `refs_bytes`, the objects reachable only
/// from the `swept_refs` refs a ref sweep deletes (both sums saturating). An upper bound, as refs_bytes is,
/// so the reason says "frees up to": the bytes, the build part with its folder count and `idle_days`, and the refs
/// part with its ref count.
pub fn sweep_figure(
    folders: &[FolderDisk],
    idle: &[PathBuf],
    idle_days: u64,
    swept_refs: usize,
    refs_bytes: u64,
) -> SweepEstimate {
    let swept: Vec<&FolderDisk> = folders.iter().filter(|f| idle.contains(&f.path)).collect();
    let build = swept.iter().fold(0u64, |sum, f| sum.saturating_add(f.build_bytes));
    let bytes = build.saturating_add(refs_bytes);
    SweepEstimate {
        bytes,
        reason: Some(format!(
            "frees up to {bytes} bytes: {build} bytes in the build dirs of {} folders idle {idle_days} days or \
             more, and up to {refs_bytes} bytes of objects reachable only from the {swept_refs} refs a ref sweep \
             deletes",
            swept.len()
        )),
    }
}
