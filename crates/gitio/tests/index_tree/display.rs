//! IndexTreeError's Display composed with GitError's Display and std::error::Error, through the public API only.
//! Runs no git and touches no files; ZITs because they run another file's code for real (S-102, rule 11).
use gitio::{GitError, IndexTreeError};
use std::path::PathBuf;

#[test]
fn git_shows_the_git_errors_text() {
    let error = GitError { args: vec!["write-tree".to_string()], code: Some(128), stderr: "fatal: oops\n".to_string() };
    assert_eq!(IndexTreeError::Git(error).to_string(), "git write-tree: exit code 128: fatal: oops");
}

#[test]
fn a_boxed_std_error_shows_the_unmerged_text() {
    let paths = ["a", "sp ace"].iter().map(PathBuf::from).collect();
    let boxed: Box<dyn std::error::Error> = Box::new(IndexTreeError::Unmerged { paths });
    assert_eq!(boxed.to_string(), "index has unmerged paths: \"a\", \"sp ace\"");
}
