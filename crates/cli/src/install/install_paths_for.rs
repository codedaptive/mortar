//! install_paths_for: The paths install writes under a HOME.

use std::path::Path;

use super::InstallPaths;

/// `.local/bin`, its `mortar` and `mr`, `.local/state/mortar` and the record under `home`.
pub fn install_paths_for(home: &Path) -> InstallPaths {
    let bin_dir = home.join(".local/bin");
    let state_dir = home.join(".local/state/mortar");
    InstallPaths {
        mortar: bin_dir.join("mortar"),
        mr: bin_dir.join("mr"),
        record: state_dir.join("install.json"),
        bin_dir,
        state_dir,
    }
}
