use crate::{GitError, TreeEntryError};

#[test]
fn git_comes_back_unchanged() {
    let error = GitError { args: vec!["ls-tree".to_string()], code: Some(128), stderr: "fatal: bad tree".to_string() };
    assert_eq!(
        GitError::from(TreeEntryError::Git(error)),
        GitError { args: vec!["ls-tree".to_string()], code: Some(128), stderr: "fatal: bad tree".to_string() }
    );
}
