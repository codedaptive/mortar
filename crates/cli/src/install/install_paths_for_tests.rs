//! install_paths_for: every path install writes, under the HOME given (spec 07 items 1 to 3 and 7).

use std::path::{Path, PathBuf};

use super::{install_paths_for, InstallPaths};

#[test]
fn every_path_is_under_the_home_given() {
    assert_eq!(
        install_paths_for(Path::new("/home/ana")),
        InstallPaths {
            bin_dir: PathBuf::from("/home/ana/.local/bin"),
            mortar: PathBuf::from("/home/ana/.local/bin/mortar"),
            mr: PathBuf::from("/home/ana/.local/bin/mr"),
            state_dir: PathBuf::from("/home/ana/.local/state/mortar"),
            record: PathBuf::from("/home/ana/.local/state/mortar/install.json"),
        }
    );
}

#[test]
fn another_home_moves_every_path() {
    let paths = install_paths_for(Path::new("/tmp/h"));
    assert_eq!(paths.mr, PathBuf::from("/tmp/h/.local/bin/mr"));
    assert_eq!(paths.record, PathBuf::from("/tmp/h/.local/state/mortar/install.json"));
}
