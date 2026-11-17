use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::world_bin_path;

#[test]
#[ignore = "acceptance: make accept"]
fn the_folder_of_mr_comes_first() {
    let path = world_bin_path(Path::new("/build/debug/mr"), Path::new("/build/debug/mortar"), OsStr::new("/usr/bin"))
        .expect("both built files in one folder");
    assert_eq!(std::env::split_paths(&path).next(), Some(PathBuf::from("/build/debug")));
}

#[test]
#[ignore = "acceptance: make accept"]
fn the_drive_path_is_kept_after_the_build_folder() {
    let (mr, mortar) = (Path::new("/build/debug/mr"), Path::new("/build/debug/mortar"));
    let path = world_bin_path(mr, mortar, OsStr::new("/usr/local/bin:/bin")).expect("both built files in one folder");
    assert_eq!(path, "/build/debug:/usr/local/bin:/bin");
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_mortar_in_another_folder_is_an_error() {
    let mr = Path::new("/build/debug/mr");
    assert!(world_bin_path(mr, Path::new("/elsewhere/mortar"), OsStr::new("/bin")).is_err());
    assert!(world_bin_path(mr, Path::new("/build/debug/deps/mortar"), OsStr::new("/bin")).is_err());
}
