//! landing_state_path: where the landing state lives, `<common>/mortar/landing.json`. The one place the path is
//! built, so read, write and remove agree on it.

use std::path::{Path, PathBuf};

/// The path of landing.json under `common`, the repository's common git dir. Touches no file.
pub(crate) fn landing_state_path(common: &Path) -> PathBuf {
    common.join("mortar").join("landing.json")
}
