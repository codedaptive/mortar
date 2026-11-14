//! The cone a repeat fills an existing folder with, from its record.

use super::FolderError;
use crate::plan::sparse_paths;
use crate::record::FolderRecord;

/// sparse_paths over the record's work-order and needed paths: their union, cleaned, sorted and deduplicated.
/// A record whose lists give no usable path is refused as Plan.
pub(crate) fn refill_cone(record: &FolderRecord) -> Result<Vec<String>, FolderError> {
    sparse_paths(&record.work_order_paths, &record.needed_paths).map_err(FolderError::Plan)
}
