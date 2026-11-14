//! source(): the inner GitError for Git, the inner MortarTrailerError for Trailers, None for each refusal.

use std::error::Error;

use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError};

use super::RereviewError;

fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

#[test]
fn git_source_is_the_inner_git_error() {
    let inner = GitError { args: vec!["range-diff".to_string()], code: Some(1), stderr: "bad".to_string() };
    let err = RereviewError::Git(inner.clone());
    assert_eq!(err.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&inner));
}

#[test]
fn trailers_source_is_the_inner_trailer_error() {
    let err = RereviewError::Trailers(MortarTrailerError::ChangeId(ChangeIdError::Malformed {
        text: "abc".to_string(),
        reason: "it has 3 chars".to_string(),
    }));
    let source = err.source().and_then(|s| s.downcast_ref::<MortarTrailerError>());
    assert!(matches!(source, Some(MortarTrailerError::ChangeId(ChangeIdError::Malformed { text, .. })) if text == "abc"));
}

#[test]
fn not_same_change_has_no_source() {
    let err = RereviewError::NotSameChange {
        reviewed: oid("1111111111111111111111111111111111111111"),
        reviewed_id: None,
        new: oid("2222222222222222222222222222222222222222"),
        new_id: None,
    };
    assert!(err.source().is_none());
}

#[test]
fn no_parent_has_no_source() {
    let err = RereviewError::NoParent { commit: oid("1111111111111111111111111111111111111111") };
    assert!(err.source().is_none());
}

#[test]
fn bases_differ_has_no_source() {
    let err = RereviewError::BasesDiffer {
        reviewed_parent: oid("cccccccccccccccccccccccccccccccccccccccc"),
        new_parent: oid("dddddddddddddddddddddddddddddddddddddddd"),
    };
    assert!(err.source().is_none());
}
