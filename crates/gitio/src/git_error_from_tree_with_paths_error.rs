//! GitError from TreeWithPathsError: the untyped form Git::tree_with_paths answers with.

use crate::{GitError, TreeWithPathsError};

/// A refusal becomes a GitError with no exit code, no args and the refusal's Display text as stderr; a GitError
/// comes back unchanged.
impl From<TreeWithPathsError> for GitError {
    fn from(error: TreeWithPathsError) -> GitError {
        match error {
            TreeWithPathsError::Refused(refusal) => GitError::refusal(Vec::new(), refusal.to_string()),
            TreeWithPathsError::Git(error) => error,
        }
    }
}
