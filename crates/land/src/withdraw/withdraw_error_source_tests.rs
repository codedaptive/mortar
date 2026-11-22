//! source(): the wrapped error for a wrapping variant; none for a refusal withdraw makes itself.

use super::WithdrawError;
use gitio::{GitError, Oid};
use std::error::Error;

fn git_error() -> GitError {
    GitError { args: vec!["update-ref".to_string()], code: Some(1), stderr: "bad".to_string() }
}

#[test]
fn a_wrapped_error_is_the_source_and_a_refusal_has_none() {
    let wrapped = WithdrawError::Git(git_error());
    assert_eq!(wrapped.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&git_error()));
    let commit = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    let origin = Oid::parse("2222222222222222222222222222222222222222").unwrap();
    let refusal = WithdrawError::Published { zone: "land".to_string(), commit, origin };
    assert!(refusal.source().is_none());
}
