//! zone_paths_record: the record a zone folder has once set_zone_paths gives it a new list of paths.
//! zone_paths_record_tests covers it.

use std::path::Path;

use super::FolderError;
use crate::record::{FolderKind, FolderRecord};

/// `existing` with its work-order paths replaced by `cone` (already cleaned by sparse_paths) and its needed paths
/// cleared, since the new list is the zone's whole set; kind, zone, request, base and parent are kept. Refuses
/// any record but a zone folder's as DifferentFolder at `folder`, carrying `existing`.
pub(crate) fn zone_paths_record(
    folder: &Path,
    existing: &FolderRecord,
    cone: &[String],
) -> Result<FolderRecord, FolderError> {
    if existing.kind != FolderKind::Zone {
        return Err(FolderError::DifferentFolder {
            path: folder.to_path_buf(),
            existing: existing.clone(),
        });
    }
    Ok(FolderRecord {
        work_order_paths: cone.to_vec(),
        needed_paths: Vec::new(),
        ..existing.clone()
    })
}
