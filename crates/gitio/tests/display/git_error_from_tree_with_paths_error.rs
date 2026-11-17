//! From<TreeWithPathsError> for GitError: each refusal stored as its Display text, with no code and no args.
use gitio::{GitError, TreeWithPathsError, TreeWithPathsRefusal};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn refusal_text(stderr: &str) -> GitError {
    GitError { args: Vec::new(), code: None, stderr: stderr.to_string() }
}

fn from(refusal: TreeWithPathsRefusal) -> GitError {
    GitError::from(TreeWithPathsError::Refused(refusal))
}

#[test]
fn a_folder_in_from_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        from(TreeWithPathsRefusal::FolderInFrom { path: PathBuf::from("src/a.rs") }),
        refusal_text("not a file entry: \"src/a.rs\" is a folder")
    );
}

#[test]
fn a_folder_in_base_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        from(TreeWithPathsRefusal::FolderInBase { path: PathBuf::from("base-dir") }),
        refusal_text("not a file entry: \"base-dir\" is a folder")
    );
}

#[test]
fn an_overlap_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        from(TreeWithPathsRefusal::Overlap { folder: PathBuf::from("lib"), inner: PathBuf::from("lib/x") }),
        refusal_text("not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"")
    );
}

#[test]
fn a_path_ending_in_a_slash_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        from(TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("dir/") }),
        refusal_text("not a usable path (ends in '/', naming a folder's children): \"dir/\"")
    );
}

#[test]
fn a_path_not_as_git_stores_it_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        from(TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("a//b") }),
        refusal_text("not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a//b\"")
    );
}

#[test]
fn an_empty_path_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        from(TreeWithPathsRefusal::Unusable { path: PathBuf::from("") }),
        refusal_text("not a usable path (empty): \"\"")
    );
}

#[test]
fn a_nul_path_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        from(TreeWithPathsRefusal::Unusable { path: PathBuf::from(OsStr::from_bytes(b"has\0nul")) }),
        refusal_text("path holds a NUL: \"has\\0nul\"")
    );
}
