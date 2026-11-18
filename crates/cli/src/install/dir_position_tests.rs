//! dir_position: the index of the first PATH entry equal to the folder, or None.

use std::path::{Path, PathBuf};

use super::dir_position;

fn entries(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

#[test]
fn the_first_equal_entry_gives_the_index() {
    let path = entries(&["/usr/bin", "/home/ana/.local/bin", "/bin", "/home/ana/.local/bin"]);
    assert_eq!(dir_position(Path::new("/home/ana/.local/bin"), &path), Some(1));
    assert_eq!(dir_position(Path::new("/usr/bin"), &path), Some(0));
}

#[test]
fn a_folder_not_on_path_is_none() {
    let path = entries(&["/usr/bin", "/home/ana/.local/bin/sub", "/home/ana/.local"]);
    assert_eq!(dir_position(Path::new("/home/ana/.local/bin"), &path), None);
    assert_eq!(dir_position(Path::new("/usr/bin"), &[]), None);
}

#[test]
fn equality_is_by_path_so_a_trailing_slash_still_matches() {
    let path = entries(&["/sbin", "/home/ana/.local/bin/"]);
    assert_eq!(dir_position(Path::new("/home/ana/.local/bin"), &path), Some(1));
}
