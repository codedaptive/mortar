//! Where a folder's record lives inside its git dir.

use std::path::{Path, PathBuf};

/// `<git_dir>/mortar-folder.json`; each worktree has its own git dir, so each folder its own record.
pub fn record_path(git_dir: &Path) -> PathBuf {
    git_dir.join("mortar-folder.json")
}
