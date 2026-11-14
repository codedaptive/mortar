//! How recently a folder was touched.
//! A bare type; idle_folders_tests exercises it.

use std::path::PathBuf;

/// A folder and the newest modification time found in it, in unix seconds; `newest` is None when nothing was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderAge {
    pub path: PathBuf,
    pub newest: Option<u64>,
}
