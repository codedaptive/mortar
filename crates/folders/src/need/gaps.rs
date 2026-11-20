//! A folder's gaps: the paths it needs that its work order does not name.
//! Pure; gaps_tests exercises it.

use crate::record::FolderRecord;

/// The needed_paths of `record` that are not in its work_order_paths, in needed_paths order.
pub fn gaps(record: &FolderRecord) -> Vec<String> {
    record
        .needed_paths
        .iter()
        .filter(|needed| !record.work_order_paths.contains(needed))
        .cloned()
        .collect()
}
