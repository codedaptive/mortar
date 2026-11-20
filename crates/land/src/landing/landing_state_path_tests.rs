//! landing_state_path: the path written out by hand for an absolute and a relative common dir.

use super::landing_state_path;
use std::path::{Path, PathBuf};

#[test]
fn absolute_common_dir_gets_mortar_landing_json_inside_it() {
    assert_eq!(landing_state_path(Path::new("/repo/.git")), PathBuf::from("/repo/.git/mortar/landing.json"));
}

#[test]
fn relative_common_dir_stays_relative() {
    assert_eq!(landing_state_path(Path::new("a/b")), PathBuf::from("a/b/mortar/landing.json"));
}
