//! absent_paths: the paths a tree lacks, from read_path_entries' answer.

use crate::Oid;
use std::path::PathBuf;

/// The paths whose entry is None, in the order given, a path given twice kept twice. `paths` and `entries` pair up
/// item by item.
pub(crate) fn absent_paths(paths: &[PathBuf], entries: &[Option<(String, Oid)>]) -> Vec<PathBuf> {
    paths.iter().zip(entries).filter(|(_, entry)| entry.is_none()).map(|(path, _)| path.clone()).collect()
}
