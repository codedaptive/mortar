use crate::{GitError, IndexTreeError};
use std::path::PathBuf;

fn unmerged(paths: &[&str]) -> IndexTreeError {
    IndexTreeError::Unmerged { paths: paths.iter().map(PathBuf::from).collect() }
}

#[test]
fn unmerged_differs_on_paths() {
    assert_ne!(unmerged(&["a"]), unmerged(&["b"]));
    assert_ne!(unmerged(&["a"]), unmerged(&["a", "b"]));
    assert_ne!(unmerged(&["a", "b"]), unmerged(&["b", "a"]));
}

#[test]
fn unmerged_is_not_a_git_error_with_its_text() {
    let git = GitError { args: Vec::new(), code: None, stderr: "index has unmerged paths: \"a\"".to_string() };
    assert_ne!(unmerged(&["a"]), IndexTreeError::Git(git));
}

#[test]
fn is_a_std_error() {
    let _boxed: Box<dyn std::error::Error> = Box::new(unmerged(&["a", "sp ace"]));
}
