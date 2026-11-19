//! The paths install writes under HOME.
//! A bare type; install_paths_for_tests exercises it.

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
