//! guard_config_path: where the guard keeps the branches it protects.

use std::path::{Path, PathBuf};

/// `<common>/mortar/guard.json`, `common` being the git dir every worktree shares, so one config serves them all.
pub(crate) fn guard_config_path(common: &Path) -> PathBuf {
    common.join("mortar").join("guard.json")
}
