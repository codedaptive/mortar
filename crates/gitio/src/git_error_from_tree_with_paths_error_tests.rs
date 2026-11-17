use crate::{GitError, TreeWithPathsError};

#[test]
fn a_git_error_comes_back_unchanged() {
    let failed = || GitError {
        args: vec!["ls-tree".to_string(), "-z".to_string()],
        code: Some(128),
        stderr: "fatal: not a tree object\n".to_string(),
    };
    assert_eq!(GitError::from(TreeWithPathsError::Git(failed())), failed());
}
