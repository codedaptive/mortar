//! repository_error: git refusing with an exit code is NotARepository naming the directory; no code is Git.

use std::path::{Path, PathBuf};

use gitio::GitError;

use super::{repository_error, ContextError};

fn git_error(code: Option<i32>) -> GitError {
    GitError { args: vec!["rev-parse".to_string()], code, stderr: "fatal".to_string() }
}

#[test]
fn an_exit_code_is_not_a_repository_naming_the_directory() {
    match repository_error(Path::new("/tmp/x"), git_error(Some(128))) {
        ContextError::NotARepository { dir, error } => {
            assert_eq!(dir, PathBuf::from("/tmp/x"));
            assert_eq!(error, git_error(Some(128)));
        }
        other => panic!("gave {other:?}"),
    }
}

#[test]
fn no_exit_code_is_git_failing() {
    match repository_error(Path::new("/tmp/x"), git_error(None)) {
        ContextError::Git(error) => assert_eq!(error, git_error(None)),
        other => panic!("gave {other:?}"),
    }
}
