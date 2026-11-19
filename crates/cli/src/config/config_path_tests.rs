//! config_path: mortar/config.json directly in the git common dir.

use std::path::{Path, PathBuf};

use super::config_path;

#[test]
fn config_is_mortar_config_json_in_the_common_dir() {
    assert_eq!(config_path(Path::new("/repo/.git")), PathBuf::from("/repo/.git/mortar/config.json"));
}

#[test]
fn relative_common_dir_stays_relative() {
    assert_eq!(config_path(Path::new(".git")), PathBuf::from(".git/mortar/config.json"));
}
