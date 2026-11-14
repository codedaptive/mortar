//! What the folders, the store and the refs cost on disk, and what a sweep would free.
//! A bare type; Nagatha's integration group covers it through disk's cases.

use crate::plan::SweepEstimate;

use super::FolderDisk;

/// Each Mortar folder; `store_bytes`, the shared object store, counted once; `refs`, the count of refs in each
/// namespace (refs_by_namespace); and `sweep`, what sweep_build would free, with what it leaves unmeasured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskReport {
    pub folders: Vec<FolderDisk>,
    pub store_bytes: u64,
    pub refs: Vec<(String, usize)>,
    pub sweep: SweepEstimate,
}
