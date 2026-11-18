//! The paths install writes under HOME (spec 07 items 1 to 3 and 7).
//! A bare type (S-7); install_paths_for_tests exercises it.

use std::path::PathBuf;

/// `bin_dir`: `~/.local/bin`. `mortar`: the placed binary, `~/.local/bin/mortar`. `mr`: the short-name link,
/// `~/.local/bin/mr`. `state_dir`: `~/.local/state/mortar`. `record`: install's record,
/// `~/.local/state/mortar/install.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPaths {
    pub bin_dir: PathBuf,
    pub mortar: PathBuf,
    pub mr: PathBuf,
    pub state_dir: PathBuf,
    pub record: PathBuf,
}
