//! source(): the inner GitError for Git, the inner io::Error for Io, None for each refusal.

use std::error::Error;
use std::io;
use std::path::PathBuf;

use gitio::GitError;

use super::GuardError;

#[test]
fn git_source_is_the_inner_git_error() {
    let inner = GitError { args: vec!["rev-parse".to_string()], code: Some(1), stderr: "bad".to_string() };
    let err = GuardError::Git(inner.clone());
    assert_eq!(err.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&inner));
}

#[test]
fn io_source_is_the_inner_io_error() {
    let err = GuardError::Io { path: PathBuf::from("/r/x"), error: io::Error::from(io::ErrorKind::PermissionDenied) };
    let source = err.source().and_then(|s| s.downcast_ref::<io::Error>());
    assert_eq!(source.map(io::Error::kind), Some(io::ErrorKind::PermissionDenied));
}

#[test]
fn foreign_hook_has_no_source() {
    assert!(GuardError::ForeignHook { path: PathBuf::from("/r/x") }.source().is_none());
}

#[test]
fn bad_config_has_no_source() {
    assert!(GuardError::BadConfig { path: PathBuf::from("/r/x"), why: "w".to_string() }.source().is_none());
}
