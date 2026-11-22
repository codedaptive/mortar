//! A folder's unsaved work, counted from git's status.
//! No unit test: it runs git; tests/folders/disk_cases.rs (the `unsaved_` cases) covers it, and count_unsaved_tests
//! the counting.

use std::path::Path;

use gitio::Git;

use super::{count_unsaved, UnsavedCounts};
use crate::make::FolderError;

/// The counts of `folder`'s unsaved work: its changed and conflicted paths, and its untracked files. It takes no
/// lock and writes nothing, as the status run is read-only. A folder whose status cannot be read is
/// Err(FolderError::Git), and a disk listing that measures that folder fails on the error. It counts one folder:
/// the disk report does not carry the counts, so a caller reads them per folder and shows them beside it.
pub fn folder_unsaved(git: &Git, folder: &Path) -> Result<UnsavedCounts, FolderError> {
    git.status_paths(folder).map(|paths| count_unsaved(&paths)).map_err(FolderError::Git)
}
