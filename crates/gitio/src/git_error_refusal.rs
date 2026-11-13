//! GitError::refusal: an error that no exit code produced.

use crate::GitError;

impl GitError {
    /// An error with no exit code whose stderr is `message`.
    pub fn refusal(args: Vec<String>, message: impl Into<String>) -> GitError {
        GitError {
            args,
            code: None,
            stderr: message.into(),
        }
    }
}
