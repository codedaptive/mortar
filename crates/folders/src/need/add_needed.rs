//! The folder record with one more needed path, or None when the record already needs it.
//! Pure; add_needed_tests exercises it.

use crate::record::FolderRecord;

/// None when `record.needed_paths` already holds `path`; else a copy of `record` with `path` added to
/// needed_paths, sorted and deduplicated. `path` is already clean (sparse_paths). A work-order path is still
/// added: the needed list records what was asked for, and gaps leaves out what the work order covers.
pub fn add_needed(record: &FolderRecord, path: &str) -> Option<FolderRecord> {
    if record.needed_paths.iter().any(|needed| needed == path) {
        return None;
    }
    let mut new = record.clone();
    new.needed_paths.push(path.to_string());
    new.needed_paths.sort();
    new.needed_paths.dedup();
    Some(new)
}
