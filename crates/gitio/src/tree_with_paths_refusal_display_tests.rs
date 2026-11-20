use crate::TreeWithPathsRefusal;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

#[test]
fn a_folder_in_from_is_not_a_file_entry() {
    let refusal = TreeWithPathsRefusal::FolderInFrom { path: PathBuf::from("src/a.rs") };
    assert_eq!(refusal.to_string(), "not a file entry: \"src/a.rs\" is a folder");
}

#[test]
fn a_folder_in_base_is_not_a_file_entry() {
    let refusal = TreeWithPathsRefusal::FolderInBase { path: PathBuf::from("src/a.rs") };
    assert_eq!(refusal.to_string(), "not a file entry: \"src/a.rs\" is a folder");
}

#[test]
fn a_quoted_and_spaced_path_shows_escaped() {
    let refusal = TreeWithPathsRefusal::FolderInFrom { path: PathBuf::from("my \"odd\" file.txt") };
    assert_eq!(refusal.to_string(), "not a file entry: \"my \\\"odd\\\" file.txt\" is a folder");
}

#[test]
fn an_overlap_names_folder_then_inner() {
    let refusal = TreeWithPathsRefusal::Overlap { folder: PathBuf::from("lib"), inner: PathBuf::from("lib/x") };
    assert_eq!(refusal.to_string(), "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"");
}

#[test]
fn a_path_ending_in_a_slash_names_a_folders_children() {
    let refusal = TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("dir/") };
    assert_eq!(refusal.to_string(), "not a usable path (ends in '/', naming a folder's children): \"dir/\"");
}

#[test]
fn a_path_not_as_git_stores_it_says_so() {
    let refusal = TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("a/../b") };
    assert_eq!(
        refusal.to_string(),
        "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a/../b\""
    );
}

#[test]
fn a_leading_slash_is_not_as_git_stores_it() {
    let refusal = TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("/a") };
    assert_eq!(
        refusal.to_string(),
        "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"/a\""
    );
}

#[test]
fn an_empty_path_is_not_usable() {
    let refusal = TreeWithPathsRefusal::Unusable { path: PathBuf::from("") };
    assert_eq!(refusal.to_string(), "not a usable path (empty): \"\"");
}

#[test]
fn a_path_holding_a_nul_says_so() {
    let refusal = TreeWithPathsRefusal::Unusable { path: PathBuf::from(OsStr::from_bytes(b"a\0b")) };
    assert_eq!(refusal.to_string(), "path holds a NUL: \"a\\0b\"");
}

#[test]
fn a_non_utf8_path_holding_a_nul_shows_lossily() {
    let refusal = TreeWithPathsRefusal::Unusable { path: PathBuf::from(OsStr::from_bytes(b"caf\xe9\0")) };
    assert_eq!(refusal.to_string(), "path holds a NUL: \"caf\u{fffd}\\0\"");
}
