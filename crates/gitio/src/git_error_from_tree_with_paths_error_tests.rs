use crate::{GitError, TreeWithPathsError, TreeWithPathsRefusal};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

#[test]
fn a_git_error_comes_back_unchanged() {
    let failed = || GitError {
        args: vec!["ls-tree".to_string(), "-z".to_string()],
        code: Some(128),
        stderr: "fatal: not a tree object\n".to_string(),
    };
    assert_eq!(GitError::from(TreeWithPathsError::Git(failed())), failed());
}

fn refusal_text(stderr: &str) -> GitError {
    GitError { args: Vec::new(), code: None, stderr: stderr.to_string() }
}

fn refused_error(refusal: TreeWithPathsRefusal) -> GitError {
    GitError::from(TreeWithPathsError::Refused(refusal))
}

#[test]
fn a_folder_in_from_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        refused_error(TreeWithPathsRefusal::FolderInFrom { path: PathBuf::from("src/a.rs") }),
        refusal_text("not a file entry: \"src/a.rs\" is a folder")
    );
}

#[test]
fn a_folder_in_base_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        refused_error(TreeWithPathsRefusal::FolderInBase { path: PathBuf::from("base-dir") }),
        refusal_text("not a file entry: \"base-dir\" is a folder")
    );
}

#[test]
fn an_overlap_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        refused_error(TreeWithPathsRefusal::Overlap { folder: PathBuf::from("lib"), inner: PathBuf::from("lib/x") }),
        refusal_text("not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"")
    );
}

#[test]
fn a_path_ending_in_a_slash_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        refused_error(TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("dir/") }),
        refusal_text("not a usable path (ends in '/', naming a folder's children): \"dir/\"")
    );
}

#[test]
fn a_path_not_as_git_stores_it_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        refused_error(TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("a//b") }),
        refusal_text("not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a//b\"")
    );
}

#[test]
fn an_empty_path_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        refused_error(TreeWithPathsRefusal::Unusable { path: PathBuf::from("") }),
        refusal_text("not a usable path (empty): \"\"")
    );
}

#[test]
fn a_nul_path_is_its_text_with_no_code_and_no_args() {
    assert_eq!(
        refused_error(TreeWithPathsRefusal::Unusable { path: PathBuf::from(OsStr::from_bytes(b"has\0nul")) }),
        refusal_text("path holds a NUL: \"has\\0nul\"")
    );
}
