//! Failure texts: Git says git trailers failed and carries git's text; ChangeId is the refusal as it is.

use gitio::GitError;

use super::MortarTrailerError;
use crate::change_id::ChangeIdError;

#[test]
fn git_says_trailers_failed_with_gits_text() {
    let err = MortarTrailerError::Git(GitError {
        args: vec!["interpret-trailers".to_string(), "--parse".to_string()],
        code: Some(128),
        stderr: "fatal: boom\n".to_string(),
    });
    assert_eq!(err.to_string(), "git trailers failed: git interpret-trailers --parse: exit code 128: fatal: boom");
}

#[test]
fn change_id_is_the_refusal_text() {
    let err = MortarTrailerError::ChangeId(ChangeIdError::Malformed {
        text: "abc".to_string(),
        reason: "it has 3 chars".to_string(),
    });
    assert_eq!(err.to_string(), "not a change ID: \"abc\": it has 3 chars");
}
