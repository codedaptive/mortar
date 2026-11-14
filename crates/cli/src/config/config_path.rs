//! Where the config lives (S-70).

use std::path::{Path, PathBuf};

/// `<common>/mortar/config.json`, `common` being the repository's git common dir, so every worktree reads
/// the same file.
pub fn config_path(common: &Path) -> PathBuf {
    common.join("mortar").join("config.json")
}
