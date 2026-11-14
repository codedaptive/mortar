use crate::{GitError, TreeWithPathsError, TreeWithPathsRefusal};
use std::path::PathBuf;

#[test]
fn a_refusal_shows_the_refusal_text() {
    let error = TreeWithPathsError::Refused(TreeWithPathsRefusal::Overlap {
        folder: PathBuf::from("f"),
        inner: PathBuf::from("f/x"),
    });
    assert_eq!(error.to_string(), "not usable paths (one is a folder holding the other): \"f\" and \"f/x\"");
}

#[test]
fn a_git_error_shows_as_git_error_does() {
    let error = TreeWithPathsError::Git(GitError {
        args: vec!["write-tree".to_string()],
        code: Some(128),
        stderr: "fatal: bad tree\n".to_string(),
    });
    assert_eq!(error.to_string(), "git write-tree: exit code 128: fatal: bad tree");
}
