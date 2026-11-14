//! guard_config_path: guard.json under the common dir's mortar folder.

use std::path::{Path, PathBuf};

use super::guard_config_path::guard_config_path;

#[test]
fn under_the_common_dirs_mortar_folder() {
    assert_eq!(guard_config_path(Path::new("/work/repo/.git")), PathBuf::from("/work/repo/.git/mortar/guard.json"));
}
