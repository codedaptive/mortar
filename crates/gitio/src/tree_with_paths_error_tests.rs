use crate::{GitError, TreeWithPathsError, TreeWithPathsRefusal};
use std::path::PathBuf;

fn folder(path: &str) -> TreeWithPathsError {
    TreeWithPathsError::Refused(TreeWithPathsRefusal::FolderInFrom { path: PathBuf::from(path) })
}

#[test]
fn is_a_std_error() {
    let _boxed: Box<dyn std::error::Error> = Box::new(folder("dir"));
}

#[test]
fn a_refusal_and_a_git_error_with_its_text_differ() {
    let git = GitError { args: Vec::new(), code: None, stderr: "not a file entry: \"dir\" is a folder".to_string() };
    assert_ne!(folder("dir"), TreeWithPathsError::Git(git));
}

#[test]
fn refusals_on_different_paths_differ() {
    assert_ne!(folder("a"), folder("b"));
}

#[test]
fn a_clone_is_equal_to_its_original() {
    assert_eq!(folder("dir").clone(), folder("dir"));
}
