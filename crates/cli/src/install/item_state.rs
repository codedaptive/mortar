//! What stands at a recorded item's path when uninstall looks.
//! A bare type; plan_uninstall_tests exercises it.

use std::path::PathBuf;

/// Gone: nothing there. File: a regular file, with content_hash of its bytes. Link: a symlink, with its target as
/// read. Dir: a folder. Other: anything else, or a path that could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemState {
    Gone,
    File { hash: String },
    Link { target: PathBuf },
    Dir,
    Other,
}
