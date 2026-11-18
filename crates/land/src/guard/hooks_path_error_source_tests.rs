//! source(): the inner GitError for Git, None for Relative.

use std::error::Error;
use std::path::PathBuf;

use gitio::GitError;

use super::HooksPathError;

#[test]
fn git_source_is_the_inner_git_error() {
    let inner = GitError { args: vec!["config".to_string()], code: Some(3), stderr: "bad".to_string() };
    let err = HooksPathError::Git(inner.clone());
    assert_eq!(err.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&inner));
}

#[test]
fn relative_has_no_source() {
    assert!(HooksPathError::Relative { value: PathBuf::from(".husky") }.source().is_none());
}
