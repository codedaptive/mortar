//! record_path: the record is mortar-folder.json directly in the folder's git dir (S-18).

use std::path::{Path, PathBuf};

use super::record_path;

#[test]
fn record_is_mortar_folder_json_in_the_git_dir() {
    assert_eq!(
        record_path(Path::new("/repo/.git/worktrees/wo-1")),
        PathBuf::from("/repo/.git/worktrees/wo-1/mortar-folder.json")
    );
}

#[test]
fn relative_git_dir_stays_relative() {
    assert_eq!(record_path(Path::new(".git")), PathBuf::from(".git/mortar-folder.json"));
}
