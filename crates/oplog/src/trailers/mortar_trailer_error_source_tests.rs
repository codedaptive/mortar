//! source(): the inner GitError for Git, the inner ChangeIdError for ChangeId.

use std::error::Error;

use gitio::GitError;

use super::MortarTrailerError;
use crate::change_id::ChangeIdError;

#[test]
fn git_source_is_the_inner_git_error() {
    let inner = GitError {
        args: vec!["interpret-trailers".to_string()],
        code: Some(1),
        stderr: "bad".to_string(),
    };
    let err = MortarTrailerError::Git(inner.clone());
    assert_eq!(err.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&inner));
}

#[test]
fn change_id_source_is_the_inner_change_id_error() {
    let err = MortarTrailerError::ChangeId(ChangeIdError::Malformed {
        text: "abc".to_string(),
        reason: "it has 3 chars".to_string(),
    });
    let text = err.source().and_then(|s| s.downcast_ref::<ChangeIdError>()).map(|e| match e {
        ChangeIdError::Malformed { text, .. } => text.as_str(),
        ChangeIdError::Random { .. } => "Random",
    });
    assert_eq!(text, Some("abc"));
}
